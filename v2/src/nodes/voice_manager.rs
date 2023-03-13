use crate::{unsafe_wrapper::UnsafeWrapper, PackedEvent};

pub struct VoiceManager<ID: PartialEq + Default + 'static, V: Default + 'static> {
    voices: Vec<VoiceContainer<ID, V>>,
    count: usize,
}

impl<ID: PartialEq + Default + 'static, V: Default + 'static> VoiceManager<ID, V> {
    pub fn new(voice_num: usize) -> Self {
        Self {
            voices: (0..voice_num)
                .map(|_| VoiceContainer {
                    id: ID::default(),
                    voice: V::default(),
                    note_off_count: 0,
                })
                .collect(),
            count: 0,
        }
    }

    pub fn note_on(&mut self, id: ID) -> &mut V {
        let i = self.next_voice_index();
        self.voices[i].id = id;
        self.voices[i].note_off_count = usize::MAX;
        &mut self.voices[i].voice
    }

    pub fn note_off(&mut self, id: ID) -> Option<&mut V> {
        for voice in &mut self.voices {
            if voice.id == id {
                voice.id = Default::default();
                voice.note_off_count = self.count;
                self.count += 1;
                return Some(&mut voice.voice);
            }
        }
        None
    }

    pub fn get_voice_mut(&mut self, id: ID) -> Option<&mut V> {
        for voice in &mut self.voices {
            if voice.id == id {
                return Some(&mut voice.voice);
            }
        }
        None
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.voices.iter_mut().map(|v| &mut v.voice)
    }

    fn next_voice_index(&mut self) -> usize {
        self.voices
            .iter()
            .enumerate()
            .min_by_key(|v| v.1.note_off_count)
            .unwrap()
            .0
    }
}

impl<ID: PartialEq + Default + Send + Sync + 'static, V: Default + Send + Sync + 'static>
    VoiceManager<ID, V>
{
    pub fn note_on_event(
        this: &UnsafeWrapper<Self>,
        id: ID,
        handler: impl FnOnce(f64, &mut V) + Send + Sync + 'static,
    ) -> PackedEvent {
        this.make_event(move |this, time| {
            handler(time, this.note_on(id));
        })
    }
}

struct VoiceContainer<ID: PartialEq + Default + 'static, V: Default + 'static> {
    id: ID,
    voice: V,
    note_off_count: usize,
}
