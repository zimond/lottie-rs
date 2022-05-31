use std::collections::{HashMap, VecDeque};

use lottie_model::{Animated, Layer, LayerContent, Model};
use slotmap::SlotMap;

use crate::layer::opacity::OpacityHierarchy;
use crate::layer::staged::{StagedLayer, TargetRef};

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
}

impl Timeline {
    pub fn set_frame_rate(&mut self, frame_rate: f32) {
        self.frame_rate = frame_rate;
    }

    pub fn items(&self) -> impl Iterator<Item = &StagedLayer> {
        self.store.values()
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
        };
        let mut layers = model
            .layers
            .iter()
            .enumerate()
            .map(|(index, layer)| LayerInfo {
                layer: layer.clone(),
                zindex: index as f32,
                child_index_window: 1.0,
                target_ref: TargetRef::Layer(layer.id),
                parent: None,
                time_remapping: layer.time_remapping(),
            })
            .collect::<VecDeque<_>>();
        let default_frame_rate = model.frame_rate;
        let mut standby_map: HashMap<u32, Vec<Id>> = HashMap::new();
        let mut parents_map = HashMap::new();
        while !layers.is_empty() {
            let LayerInfo {
                layer,
                zindex,
                child_index_window,
                target_ref,
                parent,
                time_remapping,
            } = layers.pop_front().unwrap();
            let index = layer.index;
            let parent_index = layer.parent_index;
            let mut assets = vec![];
            if let LayerContent::Precomposition(r) = &layer.content {
                let asset = match model.assets.iter().find(|asset| asset.id == r.ref_id) {
                    Some(a) => a,
                    None => continue,
                };
                let step = child_index_window / (asset.layers.len() as f32 + 1.0);
                for (index, asset_layer) in asset.layers.iter().enumerate() {
                    let asset_layer = asset_layer.clone();
                    assets.push(LayerInfo {
                        layer: asset_layer,
                        zindex: (index as f32 + 1.0) * step + zindex,
                        child_index_window: step,
                        target_ref: TargetRef::Asset(r.ref_id.clone()),
                        parent: None,
                        time_remapping: time_remapping.clone(),
                    });
                }
            }
            let mut staged = StagedLayer::new(layer);
            staged.target = target_ref;
            staged.parent = parent;
            staged.zindex = zindex;
            staged.frame_rate = default_frame_rate;
            staged.time_remapping = time_remapping;
            let id = timeline.add_item(staged);
            for mut info in assets {
                info.parent = Some(id);
                layers.push_back(info);
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

struct LayerInfo {
    layer: Layer,
    zindex: f32,
    child_index_window: f32,
    target_ref: TargetRef,
    parent: Option<Id>,
    time_remapping: Option<Animated<f32>>,
}
