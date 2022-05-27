use std::collections::{HashMap, VecDeque};

use intervaltree::{Element, IntervalTree};
use lottie_model::{LayerContent, Model};
use ordered_float::OrderedFloat;
use slotmap::SlotMap;

use crate::layer::opacity::OpacityHierarchy;
use crate::layer::staged::{StagedLayer, TargetRef};
use crate::layer::LayerExt;

slotmap::new_key_type! {
    pub struct Id;
}

#[derive(Clone)]
pub enum TimelineAction {
    Spawn(Id),
    Destroy(Id),
}

#[derive(Clone)]
pub struct Timeline {
    start_frame: f32,
    end_frame: f32,
    frame_rate: f32,
    index_id_map: HashMap<u32, Id>,
    store: SlotMap<Id, StagedLayer>,
    events: IntervalTree<OrderedFloat<f32>, TimelineAction>,
}

impl Timeline {
    pub fn set_frame_rate(&mut self, frame_rate: f32) {
        self.frame_rate = frame_rate;
    }

    fn add_item(&mut self, mut layer: StagedLayer) -> Id {
        let start_frame = layer.start_frame;
        let end_frame = layer.end_frame;
        self.start_frame = start_frame.min(self.start_frame);
        self.end_frame = end_frame.max(self.end_frame);

        let id = self.store.insert_with_key(|key| {
            layer.id = key;
            layer
        });
        id
    }

    fn build_interval_tree(&mut self) {
        self.events = IntervalTree::from_iter(self.store.iter().flat_map(|(id, layer)| {
            vec![
                (
                    layer.start_frame.into()..(layer.start_frame + 0.001).into(),
                    TimelineAction::Spawn(id),
                ),
                (
                    layer.end_frame.into()..(layer.end_frame + 0.001).into(),
                    TimelineAction::Destroy(id),
                ),
            ]
        }));
    }

    pub fn events_in(&self, start: f32, end: f32) -> impl Iterator<Item = &TimelineAction> {
        self.events
            .query(start.into()..end.into())
            .map(|element| &element.value)
    }

    pub fn item(&self, id: Id) -> Option<&StagedLayer> {
        self.store.get(id)
    }

    pub(crate) fn new(model: &Model) -> Self {
        let mut timeline = Timeline {
            start_frame: 0.0,
            end_frame: 0.0,
            frame_rate: 0.0,
            index_id_map: HashMap::new(),
            store: SlotMap::with_key(),
            events: IntervalTree::from_iter(Option::<Element<_, _>>::None.into_iter()),
        };
        let mut layers = model
            .layers
            .iter()
            .enumerate()
            .map(|(index, layer)| (layer.clone(), index, TargetRef::Layer(layer.id), None))
            .collect::<VecDeque<_>>();
        let default_frame_rate = model.frame_rate;
        let mut standby_map: HashMap<u32, Vec<Id>> = HashMap::new();
        let mut parents_map = HashMap::new();
        while !layers.is_empty() {
            let (layer, zindex, target, parent) = layers.pop_front().unwrap();
            let index = layer.index;
            let parent_index = layer.parent_index;
            let mut assets = vec![];
            if let LayerContent::Precomposition(r) = &layer.content {
                let asset = match model.assets.iter().find(|asset| asset.id == r.ref_id) {
                    Some(a) => a,
                    None => continue,
                };
                for (index, asset_layer) in asset.layers.iter().enumerate() {
                    let mut asset_layer = asset_layer.clone();
                    asset_layer.start_frame = asset_layer.start_frame.min(layer.start_frame);
                    asset_layer.end_frame = asset_layer.end_frame.min(layer.end_frame);
                    asset_layer.start_time += layer.start_time;
                    // TODO: adjust layer frame_rate
                    if asset_layer.spawn_frame() < model.end_frame {
                        assets.push((
                            asset_layer,
                            index + zindex,
                            TargetRef::Asset(r.ref_id.clone()),
                        ));
                    }
                }
            }
            let mut staged = StagedLayer::new(layer);
            staged.target = target;
            staged.parent = parent;
            staged.zindex = zindex;
            staged.frame_rate = default_frame_rate;
            let id = timeline.add_item(staged);
            for (asset, zindex, target) in assets {
                layers.push_back((asset, zindex, target, Some(id)))
            }

            if let Some(ind) = index {
                parents_map.insert(ind, id);
            }

            if let Some(index) = parent_index {
                if let Some(parent_id) = parents_map.get(&index) {
                    if let Some(child) = timeline.store.get_mut(id) {
                        child.parent = Some(*parent_id);
                    }
                } else {
                    standby_map.entry(index).or_default().push(id);
                }
            }

            if let Some(index) = index {
                for child_id in standby_map.remove(&index).into_iter().flatten() {
                    if let Some(child) = timeline.store.get_mut(child_id) {
                        child.parent = Some(id);
                    }
                }
            }
        }
        timeline.build_opacity_hierarchy();
        timeline.build_interval_tree();
        timeline
    }

    fn opacity(&self, id: Id) -> Option<OpacityHierarchy> {
        let mut layer = self.item(id)?;
        let mut stack = vec![layer.transform.opacity.clone()];
        while let Some(parent) = layer.parent {
            if let Some(l) = self.item(parent) {
                stack.push(l.transform.opacity.clone());
                layer = l;
            } else {
                break;
            }
        }
        Some(OpacityHierarchy { stack })
    }

    fn build_opacity_hierarchy(&mut self) {
        let mut result = vec![];
        for id in self.store.keys() {
            if let Some(opacity) = self.opacity(id) {
                result.push((id, opacity));
            }
        }
        for (id, opacity) in result {
            if let Some(layer) = self.store.get_mut(id) {
                layer.opacity = opacity;
            }
        }
    }
}
