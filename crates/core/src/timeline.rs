use std::cmp::{max, min};
use std::collections::VecDeque;

use lottie_ast::{LayerContent, Model};
use multimap::MultiMap;
use slotmap::SlotMap;

use crate::layer::staged::{RenderableContent, StagedLayer, TargetRef};
use crate::layer::LayerExt;

slotmap::new_key_type! {
    pub struct Id;
}

#[derive(Clone)]
pub enum TimelineAction {
    Spawn(Id),
    Destroy(Id),
}

#[derive(Default, Clone)]
pub struct Timeline {
    start_frame: u32,
    end_frame: u32,
    frame_rate: u32,
    store: SlotMap<Id, StagedLayer>,
    events: MultiMap<u32, TimelineAction>,
}

impl Timeline {
    pub fn set_frame_rate(&mut self, frame_rate: u32) {
        self.frame_rate = frame_rate;
    }

    pub fn add_item(&mut self, layer: StagedLayer) {
        let start_frame = layer.start_frame;
        let end_frame = layer.end_frame;
        self.start_frame = min(start_frame, self.start_frame);
        self.end_frame = max(end_frame, self.end_frame);

        let id = self.store.insert(layer);
        self.events.insert(start_frame, TimelineAction::Spawn(id));
        self.events.insert(end_frame, TimelineAction::Destroy(id));
    }

    pub fn events_at(&self, frame: u32) -> Option<&Vec<TimelineAction>> {
        self.events.get_vec(&frame)
    }

    pub fn item(&self, id: Id) -> Option<&StagedLayer> {
        self.store.get(id)
    }

    pub(crate) fn new(model: &Model) -> Self {
        let mut timeline = Timeline::default();
        let mut layers = model
            .layers
            .iter()
            .map(|layer| (layer.clone(), TargetRef::Layer(layer.id)))
            .collect::<VecDeque<_>>();
        let default_frame_rate = model.frame_rate;
        while !layers.is_empty() {
            let (layer, target) = layers.pop_front().unwrap();
            let start_frame = layer.spawn_frame();
            let end_frame = layer.despawn_frame();
            let layer = match layer.content {
                LayerContent::Shape(shape_group) => StagedLayer {
                    content: RenderableContent::Shape(shape_group),
                    target,
                    start_frame,
                    end_frame,
                    frame_rate: default_frame_rate,
                },
                LayerContent::Precomposition(r) => {
                    let asset = match model.assets.iter().find(|asset| asset.id == r.ref_id) {
                        Some(a) => a,
                        None => continue,
                    };
                    for asset_layer in &asset.layers {
                        let mut asset_layer = asset_layer.clone();
                        asset_layer.start_frame = min(asset_layer.start_frame, layer.start_frame);
                        asset_layer.end_frame = min(asset_layer.end_frame, layer.end_frame);
                        asset_layer.start_time += layer.start_time;
                        // TODO: adjust layer frame_rate
                        if asset_layer.spawn_frame() < model.end_frame {
                            layers.push_back((asset_layer, TargetRef::Asset(r.ref_id.clone())));
                        }
                    }
                    continue;
                }
                LayerContent::Empty => {
                    continue;
                }
                _ => todo!(),
            };
            println!(
                "add a layer start {} end {}",
                layer.start_frame, layer.end_frame
            );
            timeline.add_item(layer);
        }
        timeline
    }
}
