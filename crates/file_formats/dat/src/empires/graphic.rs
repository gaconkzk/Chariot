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
//

use crate::error::Result;

use identifier::{SlpFileId, PlayerColorId, GraphicId, SoundGroupId};
use chariot_io_tools::{ReadArrayExt, ReadExt};
use std::io::SeekFrom;

use std::io::prelude::{Read, Seek};

#[derive(Default, Debug)]
pub struct GraphicAttackSound {
    pub sound_delay: i16,
    pub sound_group_id: SoundGroupId,
}

/// Additional graphic to draw with a graphic
#[derive(Default, Debug)]
pub struct GraphicDelta {
    /// Graphic ID to draw
    pub graphic_id: GraphicId,
    /// X offset from parent graphic
    pub offset_x: i16,
    /// Y offset from parent graphic
    pub offset_y: i16,
    /// Appears to be unused in AOE 1
    display_angle: i16,
}

#[derive(Default, Debug)]
pub struct Graphic {
    pub id: GraphicId,
    pub name: String,
    pub short_name: String,
    pub slp_id: Option<SlpFileId>,
    pub layer: u8,
    pub player_color_id: Option<PlayerColorId>,
    pub second_player_color_id: Option<PlayerColorId>,

    /// Whether or not to replay at the end of the animation
    pub replay: bool,

    pub coordinates: Vec<u16>,

    /// Sound played when graphic is on screen
    pub sound_group_id: Option<SoundGroupId>,

    pub frame_count: u16,
    pub angle_count: u16,
    pub new_speed: f32,
    pub frame_rate: f32,
    pub replay_delay: f32,
    pub sequence_type: u8,
    pub mirror_mode: u8,

    /// Deltas this graphic would create. A delta is another graphic slot
    /// that would be joined to this one
    pub deltas: Vec<GraphicDelta>,

    pub attack_sounds: Vec<GraphicAttackSound>,
}

pub fn read_graphics<R: Read + Seek>(stream: &mut R) -> Result<Vec<Graphic>> {
    let mut graphics = Vec::new();
    let graphic_count = stream.read_u16()? as usize;

    // No clue what these are pointing to (the numbers are too large to be file offsets),
    // but we need to skip a graphic if one of these pointers is zero
    let graphic_pointers = stream.read_array(graphic_count, |c| c.read_u32())?;
    for graphic_pointer in &graphic_pointers {
        if *graphic_pointer == 0 {
            continue;
        }

        let mut graphic: Graphic = Default::default();
        graphic.name = stream.read_sized_str(21)?;
        graphic.short_name = stream.read_sized_str(13)?;
        graphic.slp_id = optional_id!(stream.read_i32()?);

        stream.seek(SeekFrom::Current(2))?; // skip 2 unknown bytes
        graphic.layer = stream.read_u8()?;

        graphic.player_color_id = optional_id!(stream.read_i8()?);
        graphic.second_player_color_id = optional_id!(stream.read_i8()?);
        graphic.replay = stream.read_u8()? != 0;
        graphic.coordinates = stream.read_array(4, |c| c.read_u16())?;

        let delta_count = stream.read_u16()? as usize;
        graphic.sound_group_id = optional_id!(stream.read_i16()?);
        let attack_sound_used = stream.read_u8()? as usize;
        graphic.frame_count = stream.read_u16()?;
        graphic.angle_count = stream.read_u16()?;
        graphic.new_speed = stream.read_f32()?;
        graphic.frame_rate = stream.read_f32()?;
        graphic.replay_delay = stream.read_f32()?;
        graphic.sequence_type = stream.read_u8()?;
        graphic.id = required_id!(stream.read_i16()?);
        graphic.mirror_mode = stream.read_u8()?;
        graphic.deltas = stream.read_array(delta_count, |c| read_delta(c))?
            .into_iter()
            .filter_map(|d| d)
            .collect();

        if attack_sound_used != 0 {
            // three sounds per angle
            let attack_sound_count = 3 * graphic.angle_count as usize;
            graphic.attack_sounds = stream.read_array(attack_sound_count, |c| read_attack_sound(c))?
                .into_iter()
                .filter_map(|a| a)
                .collect();
        }
        graphics.push(graphic);
    }
    Ok(graphics)
}

fn read_delta<R: Read + Seek>(stream: &mut R) -> Result<Option<GraphicDelta>> {
    let mut delta: GraphicDelta = Default::default();
    let graphic_id = optional_id!(stream.read_i16()?);
    if graphic_id.is_some() {
        delta.graphic_id = graphic_id.unwrap();
    }
    stream.seek(SeekFrom::Current(6))?; // skip unknown bytes
    delta.offset_x = stream.read_i16()?;
    delta.offset_y = stream.read_i16()?;
    delta.display_angle = stream.read_i16()?;
    stream.seek(SeekFrom::Current(2))?; // skip unknown bytes
    Ok(if graphic_id.is_some() { Some(delta) } else { None })
}

fn read_attack_sound<R: Read>(stream: &mut R) -> Result<Option<GraphicAttackSound>> {
    let mut attack_sound: GraphicAttackSound = Default::default();
    attack_sound.sound_delay = stream.read_i16()?;
    let sound_group_id = optional_id!(stream.read_i16()?);
    if sound_group_id.is_some() {
        attack_sound.sound_group_id = sound_group_id.unwrap();
    }
    Ok(if sound_group_id.is_some() { Some(attack_sound) } else { None })
}
