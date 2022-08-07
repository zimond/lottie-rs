use std::time::Duration;

use bevy::prelude::{Entity, Image, Transform};
use bevy::render::texture::{CompressedImageFormats, ImageType, TextureError};
use bevy::render::view::RenderLayers;
use bevy_tweening::{Animator, EaseMethod, Sequence, Tracks, Tween, TweeningType};
use lottie_core::prelude::*;
use lottie_core::{Transform as LottieTransform, *};
use lyon::math::Angle;
use lyon::path::path::Builder;
use lyon::path::traits::PathBuilder;
use lyon::path::Winding;
use wgpu::TextureDimension;

use crate::lens::{OpacityLens, PathLens, StrokeWidthLens, TransformLens};
use crate::material::{GradientDataStop, GradientDataUniform, GradientInfo, LottieMaterial};
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
    // pub gradient_assets: &'a mut Assets<GradientMaterial>,
    // pub gradient: &'a mut GradientManager,
    pub mask_handle: Handle<Image>,
    pub model_size: Vec2,
    pub scale: f32,
}

impl<'a> BevyStagedLayer<'a> {
    pub fn spawn(mut self, commands: &mut Commands) -> Result<Entity, TextureError> {
        let mut c = commands.spawn();
        let mut initial_transform = Transform::from_matrix(self.layer.transform.value(0.0));
        initial_transform.translation.z = self.layer.zindex as f32 * -1.0;

        log::trace!(
            "spawn layer {:?}: start {}, end {}, transform: {:?}",
            c.id(),
            self.layer.start_frame,
            self.layer.end_frame,
            initial_transform
        );
        match &self.layer.content {
            RenderableContent::Shape(shapes) => {
                let shapes = shapes.shapes();
                let count = shapes.shape_count() as f32 + 1.0;
                for (zindex, shape) in shapes.enumerate() {
                    if let Some(entity) = self.spawn_shape(
                        (zindex as f32 + 1.0) / count + self.layer.zindex as f32,
                        shape,
                        c.commands(),
                    ) {
                        log::trace!("layer {:?} get a child {:?}", c.id(), entity);
                        c.add_child(entity);
                    }
                }
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
                        false,
                    )?;
                    let handle = self.image_assets.add(image);
                    c.insert_bundle(SpriteBundle {
                        texture: handle,
                        ..Default::default()
                    });
                } else if mime.mime_type().starts_with("audio") {
                    let source = AudioSource {
                        bytes: media.content.as_slice().into(),
                    };
                    let handle = self.audio_assets.add(source);
                    c.insert(handle);
                }
            }
            RenderableContent::Group => {}
        }
        c.insert_bundle(TransformBundle {
            local: initial_transform,
            global: Default::default(),
        });
        if let Some(animator) = self.transform_animator(&self.layer.transform) {
            c.insert(animator);
        }
        let id = c.id();

        c.insert(FrameTracker(self.layer.frame_transform_hierarchy.clone()));
        c.insert_bundle(VisibilityBundle::default());
        Ok(id)
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
        let global_opacity = self.layer.opacity.initial_value();
        if global_opacity < 1.0 {
            if let Some(fill) = draw_mode.fill.as_mut() {
                fill.opacity *= global_opacity;
            }
            if let Some(stroke) = draw_mode.stroke.as_mut() {
                stroke.opacity *= global_opacity;
            }
        }

        let mut material = LottieMaterial {
            size: self.model_size * self.scale,
            mask: if self.layer.matte_mode.is_some() {
                Some(self.mask_handle.clone())
            } else {
                None
            },
            gradient: GradientDataUniform::default(),
        };

        // register gradient texture if any
        if let AnyFill::Gradient(g) = &shape.fill {
            let start = g.gradient.start.initial_value();
            let end = g.gradient.end.initial_value();
            let stops = g.gradient.colors.colors.initial_value();
            assert!(stops.len() >= 2, "gradient stops must be at least 2");
            let stops = stops.iter().map(GradientDataStop::from).collect::<Vec<_>>();
            material.gradient.stops = [stops[0].clone(), stops[1].clone()];
            material.gradient.start = Vec2::new(start.x, start.y) * self.scale;
            material.gradient.end = Vec2::new(end.x, end.y) * self.scale;
            material.gradient.use_gradient = 1;
        }
        // let stroke_index = if let AnyFill::Gradient(g) = &shape.fill {
        //     self.gradient
        //         .register(&g.gradient, self.meshes, self.gradient_assets, commands)
        //         as f32
        // } else {
        //     -1.0
        // };
        let mut transform = Transform::from_matrix(shape.transform.value(0.0));
        transform.translation.z = -1.0 * zindex;

        let mut builder = Builder::new();

        let mut c = commands.spawn();
        match &shape.shape.shape {
            Shape::Ellipse(ellipse) => {
                let Ellipse { size, position, .. } = ellipse;
                let initial_size = size.initial_value() / 2.0;
                let initial_pos = position.initial_value();
                builder.add_ellipse(
                    initial_pos.to_point(),
                    initial_size,
                    Angle::zero(),
                    Winding::Positive,
                );
                c.insert_bundle(ShapeBundle::new(builder.build(), draw_mode, transform));

                if let Some(animator) = self.transform_animator(&shape.transform) {
                    c.insert(animator);
                }
                if let Some(animator) = self.draw_mode_animator(&shape) {
                    c.insert(animator);
                }
                c.insert(LottieShapeComp(shape));
            }
            Shape::PolyStar(star) => {
                star.to_path(0.0, &mut builder);
                c.insert_bundle(ShapeBundle::new(builder.build(), draw_mode, transform));
                if let Some(animator) = self.transform_animator(&shape.transform) {
                    c.insert(animator);
                }
                if let Some(animator) = self.draw_mode_animator(&shape) {
                    c.insert(animator);
                }
            }
            Shape::Rectangle(rect) => {
                rect.to_path(0.0, &mut builder);
                c.insert_bundle(ShapeBundle::new(builder.build(), draw_mode, transform));
                if let Some(animator) = self.transform_animator(&shape.transform) {
                    c.insert(animator);
                }
                if let Some(animator) = self.draw_mode_animator(&shape) {
                    c.insert(animator);
                }
            }
            Shape::Path { d } => {
                let beziers = d.initial_value();

                beziers.to_path(0.0, &mut builder);
                c.insert_bundle(ShapeBundle::new(builder.build(), draw_mode, transform));

                if let Some(animator) = self.transform_animator(&shape.transform) {
                    c.insert(animator);
                }
                if let Some(animator) = self.draw_mode_animator(&shape) {
                    c.insert(animator);
                }

                // Add bezier tween
                if d.is_animated() {
                    let tween = d
                        .keyframes
                        .tween(self.layer.frame_rate, |start, end| PathLens { start, end });
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
        };

        if self.layer.is_mask {
            c.insert(MaskMarker).insert(RenderLayers::from_layers(&[1]));
        }

        let handle = self.material_assets.add(material);
        c.insert(handle);
        c.insert(FrameTracker(self.layer.frame_transform_hierarchy.clone()));
        Some(c.id())
    }

    fn transform_animator(&self, transform: &LottieTransform) -> Option<Animator<Transform>> {
        let mut tweens = vec![];
        let frame_rate = self.layer.frame_rate;
        if transform.is_animated() {
            tweens.push(transform.tween(frame_rate, |data, _| TransformLens { data, frames: 0.0 }));
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
                tweens.push(
                    stroke
                        .width()
                        .keyframes
                        .tween(frame_rate, |start, end| StrokeWidthLens { start, end }),
                );
            }
        }

        if self.layer.opacity.is_animated() {
            let opacity_lens = OpacityLens {
                opacity: self.layer.opacity.clone(),
                frames: self.layer.end_frame,
                fill_opacity: shape.fill.opacity().clone(),
                stroke_opacity: shape.stroke.as_ref().map(|s| s.opacity().clone()),
            };
            let secs = opacity_lens.frames as f32 / self.layer.frame_rate as f32;
            let tween = Tween::new(
                EaseMethod::Linear,
                TweeningType::Once,
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
}

#[derive(Component, Deref)]
pub struct FrameTracker(FrameTransformHierarchy);
