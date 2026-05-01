// Copyright 2026 Jhanfer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.




use crossbeam_channel::{Sender, Receiver};
use crate::{core::{files::blaze_motor::motor_structs::{FileLoadingMessage, RecursiveMessages}, runtime::{bus_structs::*, event_bus::{ChannelGroup, ChannelPair}}, system::{extended_info::extended_info_manager::ExtendedInfoMessages, sizer_manager::sizer_manager::SizerMessages}}, ui::{icons_cache::thumbnails::thumbnails_manager::ThumbnailMessages, task_manager::task_manager::TaskMessage}};


//trait y macro para el sender, ya no se usa una para cada una, send global
pub trait Routable: Sized {
    fn pair(group: &ChannelGroup) -> &ChannelPair<Self>;
    fn sender(group: &ChannelGroup) -> &Sender<Self> {
        &Self::pair(group).sender
    }
    fn receiver(group: &ChannelGroup) -> &Receiver<Self> {
        &Self::pair(group).receiver
    }
}

macro_rules! route {
    ($($ty:ty => $field:ident),* $(,)?) => {
        $(impl Routable for $ty {
            fn pair(g: &ChannelGroup) -> &ChannelPair<Self> {
                &g.$field
            }
        })*
    };
}

route! {
    FileLoadingMessage => file,
    TaskMessage => task,
    RecursiveMessages => recursive,
    UiEvent => ui,
    FileOperation => fileops,
    SizerMessages => sizer,
    ExtendedInfoMessages => extended,
    ThumbnailMessages => thumbnails,
}
