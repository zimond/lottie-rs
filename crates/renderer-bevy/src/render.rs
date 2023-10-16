use std::time::Duration;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::{Entity, Image, Transform};
use bevy::render::texture::{CompressedImageFormats, ImageType, TextureError};
use bevy::render::view::RenderLayers;
use bevy_tweening::{Animator, EaseMethod, Sequence, Tracks, Tween};
use lottie_core::prelude::{Transform as LottieTransform, *};
use lyon::math::Angle;
use lyon::path::path::Builder;
use lyon::path::Winding;

use crate::lens::{OpacityLens, PathFactoryLens, PathLens, StrokeWidthLens, TransformLens};
use crate::material::*;
use crate::plugin::MaskMarker;
use crate::shape::ShapeBundle;
use crate::tween::TweenProducer;
use crate::*;

pub struct BevyStagedLayer<'a> {
    pub layer: &'a StagedLayer,
    pub meshes: &'a mut Assets<Mesh>,
    pub image_assets: &'a mut Assets<Image>,
    pub audio_assets: &'a mut Assets<AudioSource>,
    pub material_assets: &'a mut Assets<LottieMaterial>,
    pub model_size: Vec2,
    pub scale: f32,
    pub mask_handle: Handle<Image>,
    pub mask_index: &'a mut u32,
    pub mask_count: u32,
    pub mask_registry: &'a mut HashMap<Id, u32>,
    pub zindex_window: f32,
}

impl<'a> BevyStagedLayer<'a> {
    pub fn spawn(mut self, commands: &mut Commands) -> Result<Entity, TextureError> {
        let name = self
            .layer
            .name
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Layer")
            .to_string();
        let mut c = commands.spawn(Name::new(name));
        let mut initial_transform = Transform::from_matrix(self.layer.transform.value(0.0));
        initial_transform.translation.z = self.layer.zindex as f32 * -1.0;
        if self.layer.is_mask {
            initial_transform.translation.x += (*self.mask_index as f32) * self.model_size.x;
            self.mask_registry.insert(self.layer.id, *self.mask_index);
        }

        log::trace!(
            "spawn layer {:?}: start {}, end {}, transform: {:?}",
            c.id(),
            self.layer.start_frame,
            self.layer.end_frame,
            initial_transform
        );
        match &self.layer.content {
            RenderableContent::Shape(shapes) => {
                self.spawn_shapes(&shapes, self.zindex_window, &mut c);
            }
            RenderableContent::Media(media) => {
                let mime = infer::get(&media.content).unwrap();
                if mime.mime_type().starts_with("image") {
                    initial_transform = Transform::from_matrix(
                        Transform::from_scale(Vec3::new(1.0, -1.0, 1.0))
                            .with_translation(Vec3::new(
                                media.width as f32 / 2.0,
                                media.height as f32 / 2.0,
                                0.0,
                            ))
                            .compute_matrix()
                            .mul_mat4(&initial_transform.compute_matrix()),
                    );
                    let image = Image::from_buffer(
                        &media.content,
                        ImageType::MimeType(mime.mime_type()),
                        CompressedImageFormats::NONE,
                        true,
                    )?;
                    // If the media has dimensions set, scale the image
                    let size = image.size();
                    initial_transform.scale = Vec3::new(
                        media.width as f32 / size.x,
                        media.height as f32 / size.y,
                        1.0,
                    );
                    let handle = self.image_assets.add(image);
                    let mut bundle = SpriteBundle {
                        texture: handle,
                        ..Default::default()
                    };
                    bundle.sprite.flip_x = true;
                    c.insert(bundle);
                } else if mime.mime_type().starts_with("audio") {
                    let source = AudioSource {
                        bytes: media.content.as_slice().into(),
                    };
                    let handle = self.audio_assets.add(source);
                    c.insert((
                        AudioBundle {
                            source: handle,
                            settings: PlaybackSettings {
                                mode: bevy::audio::PlaybackMode::Loop,
                                paused: true,
                                ..default()
                            },
                        },
                        LottieAudio,
                    ));
                }
            }
            RenderableContent::Group => {}
        }
        c.insert(TransformBundle {
            local: initial_transform,
            global: Default::default(),
        });
        if let Some(animator) =
            self.transform_animator(&self.layer.transform, initial_transform.translation.z, None)
        {
            c.insert(animator);
        }

        if self.layer.is_mask {
            *self.mask_index += 1;
        }

        let id = c.id();
        c.insert(FrameTracker(self.layer.frame_transform_hierarchy.clone()));
        c.insert(VisibilityBundle::default());
        Ok(id)
    }

    fn spawn_shapes(&mut self, group: &ShapeGroup, zindex_window: f32, c: &mut EntityCommands) {
        let shapes = group.styled_shapes();
        let count = shapes.shape_count() as f32 + 1.0;
        // root layers have a window of exactly 1.0
        let step = zindex_window / count;
        for (index, shape) in shapes.enumerate() {
            let zindex = index as f32 * step;
            let id = match shape.shape.shape {
                Shape::Group { shapes } => {
                    // spawn a new group
                    let mut group = c.commands().spawn(Name::new(
                        shape.shape.name.unwrap_or_else(|| String::from("Group")),
                    ));
                    group.insert(VisibilityBundle::default());

                    let mut transform = Transform::from_matrix(shape.transform.value(0.0));
                    let zindex = -1.0 * zindex;
                    transform.translation.z = zindex;
                    group.insert(TransformBundle::from_transform(transform));
                    if let Some(animator) = self.transform_animator(&shape.transform, zindex, None)
                    {
                        group.insert(animator);
                    }
                    let mut new_group = ShapeGroup { shapes };
                    // if current group has a trim, add this trim to shapes list, so it will be
                    // applied correctly
                    if !shape.trims.is_empty() {
                        for trim in &shape.trims {
                            new_group.shapes.push(ShapeLayer {
                                name: None,
                                hidden: false,
                                shape: Shape::Trim(trim.trim.clone()),
                            })
                        }
                    }
                    self.spawn_shapes(&new_group, step, &mut group);
                    Some(group.id())
                }
                _ => self.spawn_shape(zindex, shape, c.commands()),
            };
            if let Some(id) = id {
                log::trace!("layer {:?} get a child {:?}", c.id(), id);
                c.add_child(id);
            }
        }
    }

    fn spawn_shape(
        &mut self,
        zindex: f32,
        shape: StyledShape,
        commands: &mut Commands,
    ) -> Option<Entity> {
        if shape.shape.hidden {
            return None;
        }
        let mut draw_mode = utils::shape_draw_mode(&shape);
        let opacity = OpacityHierarchy::from(&self.layer.transform_hierarchy);
        let global_opacity = opacity.initial_value();
        if global_opacity < 1.0 {
            if let Some(fill) = draw_mode.fill.as_mut() {
                fill.opacity *= global_opacity;
            }
            if let Some(stroke) = draw_mode.stroke.as_mut() {
                stroke.opacity *= global_opacity;
            }
        }

        let mut material = LottieMaterial {
            size: Vec4::new(self.model_size.x, self.model_size.y, self.scale, 0.0),
            mask_info: MaskDataUniform {
                masks: [
                    UVec4::default(),
                    UVec4::default(),
                    UVec4::default(),
                    UVec4::default(),
                ],
                mask_count: self.layer.mask_hierarchy.len() as u32,
                mask_total_count: self.mask_count,
            },
            mask: if !self.layer.is_mask {
                Some(self.mask_handle.clone())
            } else {
                None
            },
            gradient: GradientDataUniform::default(),
        };

        for (index, item) in self.layer.mask_hierarchy.masks().iter().enumerate() {
            let mask_index = *self.mask_registry.get(&item.id).unwrap();
            let mode = item.mode as u32;
            material.mask_info.masks[index] = UVec4::new(mask_index, mode, 0, 0);
        }

        let mut transform = Transform::from_matrix(shape.transform.value(0.0));
        let zindex = -1.0 * zindex;
        transform.translation.z = zindex;

        let name = shape
            .shape
            .name
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Shape")
            .to_string();
        let mut c = commands.spawn(Name::new(name));

        if self.layer.is_mask {
            c.insert(MaskMarker).insert(RenderLayers::from_layers(&[1]));
        }

        let mut initial_pos = Vector2D::new(0.0, 0.0);
        let mut builder = Builder::new();

        match &shape.shape.shape {
            Shape::Ellipse(ellipse) => {
                let Ellipse { size, position, .. } = ellipse;
                let initial_size = size.initial_value() / 2.0;
                initial_pos = position.initial_value();
                builder.add_ellipse(
                    initial_pos.to_point(),
                    initial_size,
                    Angle::zero(),
                    Winding::Positive,
                );
                c.insert(ShapeBundle::new(builder.build(), draw_mode, transform));

                if let Some(animator) = self.transform_animator(&shape.transform, zindex, None) {
                    c.insert(animator);
                }
                if let Some(animator) = self.draw_mode_animator(&shape) {
                    c.insert(animator);
                }
                if let Some(animator) = self.path_animator(ellipse.clone()) {
                    c.insert(animator);
                }
            }
            Shape::PolyStar(star) => {
                initial_pos = star.position.initial_value();
                let path = star.path(0.0);
                c.insert(ShapeBundle::new(path, draw_mode, transform));
                if let Some(animator) = self.transform_animator(&shape.transform, zindex, None) {
                    c.insert(animator);
                }
                if let Some(animator) = self.draw_mode_animator(&shape) {
                    c.insert(animator);
                }
            }
            Shape::Rectangle(rect) => {
                initial_pos = rect.position.initial_value();
                let path = rect.path(0.0);
                c.insert(ShapeBundle::new(path, draw_mode, transform));
                if let Some(animator) = self.transform_animator(&shape.transform, zindex, None) {
                    c.insert(animator);
                }
                if let Some(animator) = self.draw_mode_animator(&shape) {
                    c.insert(animator);
                }
            }
            Shape::Path { d, text_range } => {
                let beziers = d.initial_value();
                let path = beziers.path(0.0);
                c.insert(ShapeBundle::new(builder.build(), draw_mode, transform));

                if let Some(animator) =
                    self.transform_animator(&shape.transform, zindex, text_range.clone())
                {
                    c.insert(animator);
                }
                if let Some(animator) = self.draw_mode_animator(&shape) {
                    c.insert(animator);
                }

                // Add bezier tween
                if d.is_animated() || !shape.trims.is_empty() {
                    let tween = d.keyframes.tween(
                        self.layer.end_frame,
                        self.layer.frame_rate,
                        |start, end, start_frame, end_frame| PathLens {
                            start,
                            end,
                            start_frame,
                            end_frame,
                            trims: shape.trims.clone(),
                        },
                    );
                    let animator = Animator::new(tween).with_state(AnimatorState::Paused);
                    c.insert(animator);
                }
            }
            Shape::Group { .. } => {
                unreachable!()
            }
            _ => {
                println!("{:?}", shape.shape.shape);
                todo!()
            }
        }

        // register gradient texture if any
        if let AnyFill::Gradient(g) = &shape.fill {
            let start = g.gradient.start.initial_value();
            let end = g.gradient.end.initial_value();
            let stops = g.gradient.colors.colors.initial_value();
            assert!(stops.len() >= 2, "gradient stops must be at least 2");
            let stops = stops.iter().map(GradientDataStop::from).collect::<Vec<_>>();
            material.gradient.stops = [stops[0].clone(), stops[1].clone()];
            material.gradient.start = Vec2::new(start.x, start.y);
            material.gradient.end = Vec2::new(end.x, end.y);
            material.gradient.use_gradient = 1;
        }
        // let stroke_index = if let AnyFill::Gradient(g) = &shape.fill {
        //     self.gradient
        //         .register(&g.gradient, self.meshes, self.gradient_assets, commands)
        //         as f32
        // } else {
        //     -1.0
        // };

        let handle = self.material_assets.add(material);
        c.insert(handle);
        c.insert(FrameTracker(self.layer.frame_transform_hierarchy.clone()));
        Some(c.id())
    }

    fn transform_animator(
        &self,
        transform: &LottieTransform,
        zindex: f32,
        text_range: Option<TextRangeInfo>,
    ) -> Option<Animator<Transform>> {
        let mut tweens = vec![];
        let frame_rate = self.layer.frame_rate;
        let mask_offset = if self.layer.is_mask {
            Vec2::new(*self.mask_index as f32 * self.model_size.x, 0.0)
        } else {
            Vec2::ZERO
        };
        if transform.is_animated() || text_range.is_some() {
            let mut frames = transform.frames();
            if text_range.is_some() {
                frames = frames.max(self.layer.end_frame);
            }
            let secs = frames as f32 / frame_rate as f32;
            let transform = TransformLens {
                data: transform.clone(),
                zindex,
                frames,
                mask_offset,
                text_range: text_range.clone(),
            };
            let tween = Tween::new(
                EaseMethod::Linear,
                Duration::from_secs_f32(secs.max(f32::EPSILON)),
                transform,
            );
            tweens.push(Sequence::from_single(tween));
        }
        if !tweens.is_empty() {
            let tracks = Tracks::new(tweens);
            Some(Animator::new(tracks).with_state(AnimatorState::Paused))
        } else {
            None
        }
    }

    fn draw_mode_animator(&self, shape: &StyledShape) -> Option<Animator<DrawMode>> {
        let mut tweens = vec![];
        let frame_rate = self.layer.frame_rate;
        if let Some(stroke) = shape.stroke.as_ref() {
            if stroke.width().is_animated() {
                tweens.push(stroke.width().keyframes.tween(
                    self.layer.end_frame,
                    frame_rate,
                    |start, end, _, _| StrokeWidthLens { start, end },
                ));
            }
        }

        let opacity = OpacityHierarchy::from(&self.layer.transform_hierarchy);
        if opacity.is_animated() {
            let opacity_lens = OpacityLens {
                opacity,
                frames: self.layer.end_frame,
                fill_opacity: shape.fill.opacity().clone(),
                stroke_opacity: shape.stroke.as_ref().map(|s| s.opacity().clone()),
            };
            let secs =
                (opacity_lens.frames as f32 / self.layer.frame_rate as f32).max(f32::EPSILON);
            let tween = Tween::new(
                EaseMethod::Linear,
                Duration::from_secs_f32(secs),
                opacity_lens,
            );
            tweens.push(Sequence::from_single(tween));
        }

        if !tweens.is_empty() {
            let tracks = Tracks::new(tweens);
            Some(Animator::new(tracks).with_state(AnimatorState::Paused))
        } else {
            None
        }
    }

    fn path_animator(
        &self,
        factory: impl PathFactory + Send + Sync + 'static,
    ) -> Option<Animator<Path>> {
        let frames = self.layer.end_frame - self.layer.start_frame;
        let secs = frames / self.layer.frame_rate;
        Some(Animator::new(Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(secs),
            PathFactoryLens {
                start_frame: self.layer.start_frame,
                end_frame: self.layer.end_frame,
                factory: Box::new(factory),
            },
        )))
    }
}

#[derive(Component, Deref)]
pub struct FrameTracker(FrameTransformHierarchy);

#[derive(Component)]
pub struct LottieAudio;
