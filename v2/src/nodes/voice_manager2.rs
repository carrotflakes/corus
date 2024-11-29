pub struct VoiceManager<ID: PartialEq + Default + 'static> {
    voices: Vec<Voice<ID>>,
    count: usize,
}

struct Voice<ID> {
    id: ID,
    note_off_count: usize,
}

impl<ID: PartialEq + Default + 'static> VoiceManager<ID> {
    pub fn new(voice_num: usize) -> Self {
        Self {
            voices: (0..voice_num)
                .map(|_| Voice {
                    id: ID::default(),
                    note_off_count: 0,
                })
                .collect(),
            count: 0,
        }
    }

    pub fn note_on(&mut self, id: ID) -> usize {
        let i = self.next_voice_index();
        self.voices[i].id = id;
        self.voices[i].note_off_count = usize::MAX;
        i
    }

    pub fn note_off(&mut self, id: ID) -> Option<usize> {
        for (i, voice) in self.voices.iter_mut().enumerate() {
            if voice.id == id {
                voice.id = Default::default();
                voice.note_off_count = self.count;
                self.count += 1;
                return Some(i);
            }
        }
        None
    }

    pub fn get_index_by_id(&self, id: ID) -> Option<usize> {
        for (i, voice) in self.voices.iter().enumerate() {
            if voice.id == id {
                return Some(i);
            }
        }
        None
    }

    pub fn voice_num(&self) -> usize {
        self.voices.len()
    }

    pub fn set_voice_num(&mut self, voice_num: usize) {
        self.voices.resize_with(voice_num, || Voice {
            id: ID::default(),
            note_off_count: 0,
        });
    }

    fn next_voice_index(&self) -> usize {
        self.voices
            .iter()
            .enumerate()
            .min_by_key(|v| v.1.note_off_count)
            .unwrap()
            .0
    }
}
