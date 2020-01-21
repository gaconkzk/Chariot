// Chariot: An open source reimplementation of Age of Empires (1997)
// Copyright (c) 2016 Kevin Fuller
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use crate::ecs::{CameraComponent, TransformComponent};
use crate::ecs::resource::Viewport;
use nalgebra::Vector2;
use specs::{self, Join};
use super::System;
use types::Fixed;

pub struct CameraPositionSystem;

impl CameraPositionSystem {
    pub fn new() -> CameraPositionSystem {
        CameraPositionSystem
    }
}

impl System for CameraPositionSystem {
    fn update(&mut self, arg: specs::RunArg, _time_step: Fixed) {
        fetch_components!(arg, _entities, [
            components(transforms: TransformComponent),
            components(cameras: CameraComponent),
            mut resource(viewport: Viewport),
        ]);

        // Grab camera position from first encountered enabled camera
        for (transform, _camera) in (&transforms, &cameras).iter() {
            let position = transform.position();
            viewport.set_top_left(Vector2::new(position.x.into(), position.y.into()));
            break;
        }
    }
}
