//! Procedurally generated languages. See <http://mewo2.com/notes/naming-language/> for the
//! approach.

use rand::Rng;
use std::cmp;
use std::collections::BTreeMap;

/// The number of "extra" morphemes to consider when picking one.
const EXTRA_MORPHEMES: usize = 10;

/// The maximum character length of a given name.
const MAX_GIVEN_LEN: usize = 12;

/// A language is a collection of morphemes, and rules for generating more morphemes.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Language {
    // Morphemes
    /// Generic morphemes
    generic_morphemes: Vec<String>,
    /// Place morphemes.
    place_morphemes: Vec<String>,
    /// Region morphemes.
    region_morphemes: Vec<String>,
    /// Name morphemes
    name_morphemes: Vec<String>,
    /// Particle morpehems
    particle_morphemes: Vec<String>,

    // Names
    /// Range of morphemes in a given name (inclusive)
    per_given: (usize, usize),
    /// Structure of surnames.
    surname_structure: Vec<(N, bool)>,
    /// Probability that a particle morpheme gets capitalised.
    capitalise_particles: f64,
    /// Joiner to connect particles and double-barreled bits to the rest of the name.
    joiner: char,

    // Words
    /// Range of morphemes in a word (inclusive)
    per_word: (usize, usize),
    /// Place names
    place_words: Vec<String>,
    /// Region names
    region_words: Vec<String>,

    // Syllable / morpheme generation
    /// The available vowels
    vowels: Vec<char>,
    /// The available consonants.
    consonants: Vec<char>,
    /// The available sibilants.
    sibilants: Vec<char>,
    /// The available liquids.
    liquids: Vec<char>,
    /// The available finals.
    finals: Vec<char>,
    /// How letters are romanized.
    orthography: BTreeMap<char, String>,
    /// Structure of syllables.
    syllable_structure: Vec<(L, bool)>,
}

/// Types of letters
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum L {
    /// Vowel
    V,
    /// Consonant
    C,
    /// Sibilant
    S,
    /// Liquid
    L,
    /// Final
    F,
}

/// Types of name components
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum N {
    /// Generic morpheme
    G,
    /// Region morpheme
    R,
    /// Name morpheme
    N,
    /// Particle morpheme
    P,
    /// Inter-morpheme space
    S,
}

impl Language {
    /// Generate a random language.
    pub fn new<R: Rng>(rng: &mut R) -> Language {
        let mut l = Language {
            generic_morphemes: Vec::new(),
            place_morphemes: Vec::new(),
            region_morphemes: Vec::new(),
            name_morphemes: Vec::new(),
            particle_morphemes: Vec::new(),
            per_given: (0, 0),
            surname_structure: Vec::new(),
            capitalise_particles: 0.0,
            joiner: ' ',
            per_word: (0, 0),
            place_words: Vec::new(),
            region_words: Vec::new(),
            vowels: Vec::new(),
            consonants: Vec::new(),
            sibilants: Vec::new(),
            liquids: Vec::new(),
            finals: Vec::new(),
            orthography: BTreeMap::new(),
            syllable_structure: Vec::new(),
        };

        // Letter sets
        l.vowels = match rng.gen_range(0, 7) {
            0 => vec!['a', 'e', 'i', 'o', 'u'],
            1 => vec!['a', 'i', 'u'],
            2 => vec!['a', 'e', 'i', 'o', 'u', 'A', 'E', 'I'],
            3 => vec!['a', 'e', 'i', 'o', 'u', 'U'],
            4 => vec!['a', 'i', 'u', 'A', 'I'],
            5 => vec!['e', 'o', 'u'],
            _ => vec!['a', 'e', 'i', 'o', 'u', 'A', 'O', 'U'],
        };
        l.consonants = match rng.gen_range(0, 8) {
            0 => vec!['p', 't', 'k', 'm', 'n', 'l', 's'],
            1 => vec!['p', 't', 'k', 'b', 'd', 'g', 'm', 'n', 'l', 'r', 's', 'ʃ', 'z', 'ʒ', 'ʧ'],
            2 => vec!['p', 't', 'k', 'm', 'n', 'h'],
            3 => vec!['h', 'k', 'l', 'm', 'n', 'p', 'w'],
            4 => vec!['p', 't', 'k', 'q', 'v', 's', 'g', 'r', 'm', 'n', 'ŋ', 'l', 'j'],
            5 => vec!['t', 'k', 's', 'ʃ', 'd', 'b', 'q', 'ɣ', 'x', 'm', 'n', 'l', 'r', 'w', 'j'],
            6 => vec!['t', 'k', 'd', 'g', 'm', 'n', 's', 'ʃ'],
            _ => vec!['p', 't', 'k', 'b', 'd', 'g', 'm', 'n', 's', 'z', 'ʒ', 'ʧ', 'h', 'j', 'w'],
        };
        l.sibilants = match rng.gen_range(0, 3) {
            0 => vec!['s'],
            1 => vec!['s', 'ʃ'],
            _ => vec!['s', 'ʃ', 'f'],
        };
        l.liquids = match rng.gen_range(0, 5) {
            0 => vec!['r'],
            1 => vec!['l'],
            2 => vec!['r', 'l'],
            3 => vec!['w', 'j'],
            _ => vec!['r', 'l', 'w', 'j'],
        };
        l.finals = match rng.gen_range(0, 4) {
            0 => vec!['m', 'n'],
            1 => vec!['s', 'k'],
            2 => vec!['m', 'n', 'ŋ'],
            _ => vec!['s', 'ʃ', 'z', 'ʒ'],
        };
        rng.shuffle(&mut l.vowels);
        rng.shuffle(&mut l.consonants);
        rng.shuffle(&mut l.sibilants);
        rng.shuffle(&mut l.liquids);
        rng.shuffle(&mut l.finals);

        // Orthography
        let _ = l.orthography.insert('ʃ', "sh".to_string());
        let _ = l.orthography.insert('ʒ', "zh".to_string());
        let _ = l.orthography.insert('ʧ', "ch".to_string());
        let _ = l.orthography.insert('ʤ', "j".to_string());
        let _ = l.orthography.insert('ŋ', "ng".to_string());
        let _ = l.orthography.insert('j', "j".to_string());
        let _ = l.orthography.insert('x', "kh".to_string());
        let _ = l.orthography.insert('ɣ', "gh".to_string());
        match rng.gen_range(0, 3) {
            0 => {
                let _ = l.orthography.insert('ʃ', "sch".to_string());
                let _ = l.orthography.insert('ʒ', "zh".to_string());
                let _ = l.orthography.insert('ʧ', "tcsh".to_string());
                let _ = l.orthography.insert('ʤ', "dz".to_string());
                let _ = l.orthography.insert('x', "ch".to_string());
            }
            1 => {
                let _ = l.orthography.insert('ʃ', "ch".to_string());
                let _ = l.orthography.insert('ʒ', "j".to_string());
                let _ = l.orthography.insert('ʧ', "tch".to_string());
                let _ = l.orthography.insert('ʤ', "dj".to_string());
                let _ = l.orthography.insert('j', "y".to_string());
            }
            _ => {
                let _ = l.orthography.insert('ʃ', "x".to_string());
                let _ = l.orthography.insert('ʧ', "q".to_string());
                let _ = l.orthography.insert('j', "y".to_string());
            }
        }
        match rng.gen_range(0, 5) {
            0 => {
                let _ = l.orthography.insert('A', "á".to_string());
                let _ = l.orthography.insert('E', "é".to_string());
                let _ = l.orthography.insert('I', "í".to_string());
                let _ = l.orthography.insert('O', "ó".to_string());
                let _ = l.orthography.insert('U', "ú".to_string());
            }
            1 => {
                let _ = l.orthography.insert('A', "ä".to_string());
                let _ = l.orthography.insert('E', "ë".to_string());
                let _ = l.orthography.insert('I', "ï".to_string());
                let _ = l.orthography.insert('O', "ö".to_string());
                let _ = l.orthography.insert('U', "ü".to_string());
            }
            2 => {
                let _ = l.orthography.insert('A', "â".to_string());
                let _ = l.orthography.insert('E', "ê".to_string());
                let _ = l.orthography.insert('I', "y".to_string());
                let _ = l.orthography.insert('O', "ô".to_string());
                let _ = l.orthography.insert('U', "w".to_string());
            }
            3 => {
                let _ = l.orthography.insert('A', "au".to_string());
                let _ = l.orthography.insert('E', "ei".to_string());
                let _ = l.orthography.insert('I', "ie".to_string());
                let _ = l.orthography.insert('O', "ou".to_string());
                let _ = l.orthography.insert('U', "oo".to_string());
            }
            _ => {
                let _ = l.orthography.insert('A', "aa".to_string());
                let _ = l.orthography.insert('E', "ee".to_string());
                let _ = l.orthography.insert('I', "ii".to_string());
                let _ = l.orthography.insert('O', "oo".to_string());
                let _ = l.orthography.insert('U', "uu".to_string());
            }
        }

        // Structure
        l.syllable_structure = match rng.gen_range(0, 22) {
            0 => vec![(L::C, true), (L::V, true), (L::C, true)],
            1 => vec![(L::C, true), (L::V, true), (L::V, false), (L::C, true)],
            2 => vec![(L::C, true), (L::V, true), (L::V, true), (L::C, false)],
            3 => vec![(L::C, true), (L::V, true), (L::C, false)],
            4 => vec![(L::C, true), (L::V, true)],
            5 => vec![(L::V, true), (L::C, true)],
            6 => vec![(L::C, true), (L::V, true), (L::F, true)],
            7 => vec![(L::C, false), (L::V, true), (L::C, true)],
            8 => vec![(L::C, true), (L::V, true), (L::F, false)],
            9 => vec![(L::C, true), (L::L, false), (L::V, true), (L::C, true)],
            10 => vec![(L::C, true), (L::L, false), (L::V, true), (L::F, true)],
            11 => vec![(L::S, false), (L::C, true), (L::V, true), (L::C, true)],
            12 => vec![(L::S, false), (L::C, true), (L::V, true), (L::F, true)],
            13 => vec![(L::S, false), (L::C, true), (L::V, true), (L::C, false)],
            15 => vec![(L::C, false), (L::V, true), (L::F, true)],
            16 => vec![(L::C, false), (L::V, true), (L::C, false)],
            17 => vec![(L::C, false), (L::V, true), (L::F, false)],
            18 => vec![(L::C, false), (L::L, false), (L::V, true), (L::C, true)],
            19 => vec![(L::C, true), (L::V, true), (L::L, false), (L::C, false)],
            20 => vec![(L::C, false), (L::V, true), (L::L, false), (L::C, true)],
            21 => vec![(L::C, true), (L::V, true), (L::S, false), (L::V, true)],
            _ => vec![(L::C, false), (L::V, true), (L::L, true), (L::C, false)],
        };

        // Length
        l.per_word.0 = rng.gen_range(1, 3);
        if l.syllable_structure.len() < 3 {
            l.per_word.0 += 1;
        }
        l.per_word.1 = rng.gen_range(l.per_word.0 + 1, 6);

        // Names
        l.per_given.0 = rng.gen_range(1, 3);
        l.per_given.1 = cmp::min(l.per_word.1, rng.gen_range(l.per_given.0 + 1, 6));
        l.surname_structure = match rng.gen_range(0, 7) {
            0 => vec![(N::N, true), (N::G, false), (N::N, false)],
            1 => vec![(N::P, false), (N::P, true), (N::R, true)],
            2 => vec![(N::G, true), (N::G, true), (N::N, true)],
            3 => vec![(N::G, true), (N::P, false), (N::G, true)],
            4 => vec![(N::G, true), (N::S, true), (N::R, true)],
            5 => vec![(N::G, true), (N::S, false), (N::G, true), (N::S, false), (N::N, true)],
            _ => vec![(N::G, true), (N::S, false), (N::R, true)],
        };
        l.capitalise_particles = rng.next_f64();
        l.joiner = *rng.choose(&[' ', ' ', ' ', ' ', '-', '-', '\'']).unwrap();

        l
    }

    /// Generate a personal name.
    pub fn gen_personal<R: Rng>(&mut self, rng: &mut R) -> String {
        // Given name
        let mut given = "".to_string();
        loop {
            given = "".to_string();
            let glen = rng.gen_range(self.per_given.0, self.per_given.1 + 1);
            let nidx = rng.gen_range(0, glen);
            for i in 0..glen {
                let (morph, new) = self.pick_morpheme(rng,
                                                      if i == nidx {
                                                          &self.name_morphemes
                                                      } else {
                                                          &self.generic_morphemes
                                                      });
                if new {
                    if i == nidx {
                        self.name_morphemes.push(morph.clone());
                    } else {
                        self.generic_morphemes.push(morph.clone());
                    }
                }
                given += morph.as_str();
            }

            // Check the length limit
            if given.len() <= MAX_GIVEN_LEN {
                break;
            }
        }
        given = capitalise_first(given);

        // Surname
        let mut surname = "".to_string();
        let mut particle = false;
        let mut first = true;
        for &(ref ty, req) in &self.surname_structure {
            if !req && rng.gen() {
                continue;
            }
            if particle {
                surname += self.joiner.to_string().as_str();
            }
            let piece = match ty {
                &N::G => {
                    particle = false;
                    let (morph, new) = self.pick_morpheme(rng, &self.generic_morphemes);
                    if new {
                        self.generic_morphemes.push(morph.clone())
                    }
                    if first {
                        first = false;
                        capitalise_first(morph)
                    } else {
                        morph
                    }
                }
                &N::R => {
                    particle = false;
                    let (morph, new) = self.pick_morpheme(rng, &self.region_morphemes);
                    if new {
                        self.region_morphemes.push(morph.clone())
                    }
                    if first {
                        first = false;
                        capitalise_first(morph)
                    } else {
                        morph
                    }
                }
                &N::N => {
                    particle = false;
                    let (morph, new) = self.pick_morpheme(rng, &self.name_morphemes);
                    if new {
                        self.name_morphemes.push(morph.clone())
                    }
                    if first {
                        first = false;
                        capitalise_first(morph)
                    } else {
                        morph
                    }
                }
                &N::P => {
                    if !first && !particle {
                        surname += self.joiner.to_string().as_str();
                    }
                    particle = true;
                    let (morph, new) = self.pick_morpheme(rng, &self.particle_morphemes);
                    if new {
                        let cmorph = if rng.next_f64() < self.capitalise_particles {
                            capitalise_first(morph)
                        } else {
                            morph
                        };
                        self.particle_morphemes.push(cmorph.clone());
                        cmorph
                    } else {
                        morph
                    }
                }
                &N::S => {
                    first = true;
                    if self.joiner == '\'' {
                            '-'
                        } else {
                            self.joiner
                        }
                        .to_string()
                }
            };
            surname += piece.as_str();
        }

        format!("{} {}", given, surname)
    }

    /// Generate a place name
    pub fn gen_place<R: Rng>(&mut self, rng: &mut R) -> String {
        "".to_string()
    }

    /// Generate a region name
    pub fn gen_region<R: Rng>(&mut self, rng: &mut R) -> String {
        "".to_string()
    }

    /// Pick a morpheme from a vector, possibly generating a new one. The return value `true`
    /// indicates the produced morpheme is new and should be inserted into the vector.
    fn pick_morpheme<R: Rng>(&self, rng: &mut R, morphemes: &Vec<String>) -> (String, bool) {
        let i = rng.gen_range(0, morphemes.len() + EXTRA_MORPHEMES);
        if i < morphemes.len() {
            (morphemes[i].clone(), false)
        } else {
            match self.gen_morpheme(rng) {
                Some(morph) => (morph, true),
                None => (rng.choose(morphemes).unwrap().clone(), false),
            }
        }
    }

    /// Generate a fresh morpheme. If this fails to generate a fresh one in 1000 tries, it gives up.
    fn gen_morpheme<R: Rng>(&self, rng: &mut R) -> Option<String> {
        for _ in 0..1000 {
            let mut chs = Vec::new();

            // Generate characters
            for &(ref ty, req) in &self.syllable_structure {
                if !req && rng.gen() {
                    continue;
                }
                chs.push(choose(rng,
                                match ty {
                                    &L::V => &self.vowels,
                                    &L::C => &self.consonants,
                                    &L::S => &self.sibilants,
                                    &L::L => &self.liquids,
                                    &L::F => &self.finals,
                                }))
            }

            // Apply orthography
            let mut morph = "".to_string();
            for c in chs {
                morph += self.orthography.get(&c).unwrap_or(&c.to_string()).as_str()
            }

            // Check if it's a duplicate
            if self.generic_morphemes.contains(&morph) ||
               self.particle_morphemes.contains(&morph) ||
               self.place_morphemes.contains(&morph) ||
               self.region_morphemes.contains(&morph) ||
               self.name_morphemes.contains(&morph) {
                continue;
            }

            // Done!
            return Some(morph);
        }

        None
    }
}

/// Capitalise the first letter of a string.
fn capitalise_first(s: String) -> String {
    let mut c = s.as_str().chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Choose a value from a vector, biassed towards the start.
fn choose<'a, R: Rng, X>(rng: &mut R, xs: &'a Vec<X>) -> &'a X {
    &xs[(rng.next_f64() * xs.len() as f64).floor() as usize]
}
