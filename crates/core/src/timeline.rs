use std::collections::{HashMap, VecDeque};

use lottie_model::{Animated, Asset, Layer, LayerContent, Model, Shape};
use slotmap::SlotMap;

use crate::font::FontDB;
use crate::layer::frame::{FrameInfo, FrameTransformHierarchy};
use crate::layer::hierarchy::TransformHierarchy;
use crate::layer::staged::{StagedLayer, TargetRef};
use crate::prelude::RenderableContent;
use crate::Error;

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

    pub fn gradient_count(&self) -> usize {
        self.items().fold(0, |current, item| {
            current
                + match &item.content {
                    RenderableContent::Shape(shape_group) => shape_group
                        .shapes
                        .iter()
                        .filter(|shape| match &shape.shape {
                            Shape::GradientFill(_) | Shape::GradientStroke(_) => true,
                            _ => false,
                        })
                        .count(),
                    _ => 0,
                }
        })
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

    pub(crate) fn new(model: &Model, fontdb: &FontDB) -> Result<Self, Error> {
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
        let mut previous = None;
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
            match &layer.content {
                LayerContent::PreCompositionRef(r) => {
                    match model.assets.iter().find(|asset| asset.id() == r.ref_id) {
                        Some(Asset::Precomposition(asset)) => {
                            let step = child_index_window / (asset.layers.len() as f32 + 1.0);
                            for (index, asset_layer) in asset.layers.iter().enumerate() {
                                let asset_layer = asset_layer.clone();

                                assets.push(LayerInfo {
                                    layer: asset_layer,
                                    zindex: (index as f32 + 1.0) * step + zindex,
                                    child_index_window: step,
                                    target_ref: TargetRef::Asset(r.ref_id.clone()),
                                    parent: None,
                                    time_remapping: None,
                                });
                            }
                        }
                        _ => continue,
                    }
                }
                LayerContent::MediaRef(i) => {
                    match model.assets.iter().find(|asset| asset.id() == i.ref_id) {
                        Some(Asset::Media(media)) => {
                            let content = LayerContent::Media(media.clone());
                            let layer = Layer::new(
                                content,
                                layer.start_frame,
                                layer.end_frame,
                                layer.start_time,
                            );
                            assets.push(LayerInfo {
                                layer,
                                zindex: zindex + 0.5,
                                child_index_window: 0.5,
                                target_ref: TargetRef::Asset(i.ref_id.clone()),
                                parent: None,
                                time_remapping: None,
                            });
                        }
                        _ => continue,
                    }
                }
                _ => {}
            }

            let mut staged = StagedLayer::new(layer, model, fontdb)?;
            staged.target = target_ref;
            staged.parent = parent;
            staged.zindex = zindex;
            staged.frame_rate = default_frame_rate;
            staged.frame_transform.time_remapping = time_remapping;
            staged.frame_transform.frame_rate = default_frame_rate;

            if let Some(id) = previous {
                if staged.matte_mode.is_some() {
                    timeline.store.get_mut(id).unwrap().is_mask = true;
                }
            }

            let id = timeline.add_item(staged);
            previous = Some(id);
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
        timeline.build_frame_hierarchy();
        Ok(timeline)
    }

    fn transform_hierarchy(&self, id: Id) -> Option<TransformHierarchy> {
        let mut layer = self.item(id)?;
        let mut stack = vec![layer.transform.clone()];
        while let Some(parent) = layer.parent {
            if let Some(l) = self.item(parent) {
                stack.push(l.transform.clone());
                layer = l;
            } else {
                break;
            }
        }
        Some(TransformHierarchy { stack })
    }

    fn build_opacity_hierarchy(&mut self) {
        let mut result = vec![];
        for id in self.store.keys() {
            if let Some(t) = self.transform_hierarchy(id) {
                result.push((id, t));
            }
        }
        for (id, t) in result {
            if let Some(layer) = self.store.get_mut(id) {
                layer.transform_hierarchy = t;
            }
        }
    }

    /// This could possibly be omitted when https://github.com/bevyengine/bevy/issues/3874 is fixed
    fn build_frame_hierarchy(&mut self) {
        let ids = self.store.keys().collect::<Vec<_>>();
        for id in ids {
            let mut layer = self.store.get(id).unwrap();
            let mut stack = vec![FrameInfo {
                start_frame: layer.start_frame,
                end_frame: layer.end_frame,
                frame_transform: layer.frame_transform.clone(),
            }];
            while let Some(parent) = layer.parent.and_then(|id| self.store.get(id)) {
                stack.push(FrameInfo {
                    start_frame: parent.start_frame,
                    end_frame: parent.end_frame,
                    frame_transform: parent.frame_transform.clone(),
                });
                layer = parent;
            }
            stack.reverse();
            self.store.get_mut(id).unwrap().frame_transform_hierarchy =
                FrameTransformHierarchy { stack };
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
