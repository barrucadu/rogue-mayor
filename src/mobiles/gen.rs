//! Random mob generator. Generation works by randomly picking a target age, with some limitations
//! (adventurers *tend* to be younger); and producing personality traits by simulating (to a rough
//! measure) the entire life so far.
//!
//! Life simulation is done by applying "training packages", an idea I took from the Ars Magica 5
//! Grog sourcebook. Grogs are minor characters, so they don't need to have much thought put into
//! their creation, the grog book eases this by providing 3- and 5-year "training packages", which
//! give a result of n-years of some role, such as a scout, or a farmer. Training packages used here
//! are constrained somewhat, with packages having relations and pre-requisites. This is used to
//! help ensure that someone's life is a bit more realistic: for instance, someone isn't going to be
//! a soldier for 5 years, then a scholar for 3, then a farmer for 7 (well, they *could*, but it'd
//! be weird).

use mobiles::{LifeEvent, Mobile};
use rand::Rng;
use rand::distributions::{ChiSquared, IndependentSample, Normal};
use std::cmp;
use std::collections::BTreeMap;

/// The minimum age, and the length of the early childhood training packages.
const MIN_AGE: usize = 5;

/// The age at which a child becomes an adult.
const ADULT_AGE: usize = 13;

/// The minimum age of an adventurer.
const MIN_ONSET: usize = 20;

impl Mobile {
    /// Generate an adventurer.
    pub fn gen_adventurer<R: Rng>(rng: &mut R) -> Mobile {
        // Adventurers *tend* to be young. So use MIN_ONSET + a chi-squared(10) distribution. This
        // will give a typical age of ~(MIN_ONSET + 7), but there'll still be some older guys. For
        // in-world motivation, adventurers tend to be young because they (a) need to be physically
        // fit; and (b) tend to die.
        let chi = ChiSquared::new(10.0);
        let age = (chi.ind_sample(rng) + MIN_ONSET as f64).round() as usize;
        gen(rng, age, true)
    }

    /// Generate a child.
    pub fn gen_child<R: Rng>(rng: &mut R) -> Mobile {
        // Children are by necessity young, but far younger than an adventurer. I could skew this
        // distribution by thinking about childhood mortality, but that seems a bit dark. So let's
        // just have a uniform selection and say that childhood ends at ADULT_AGE. The minimum age
        // is MIN_AGE, as that is the length of the early childhood training packages.
        let age = rng.gen_range(MIN_AGE, ADULT_AGE);
        gen(rng, age, false)
    }

    /// Generate an adult.
    pub fn gen_adult<R: Rng>(rng: &mut R) -> Mobile {
        // Being a non-adventurer is safer than being an adventurer, so we don't get the same
        // tail-off in age as with adventurers. Some adults are old, some adults are young, some are
        // middle-aged; so let's go for a normal distribution!
        let ufm = Normal::new(30.0, 5.0);
        let age = ufm.ind_sample(rng).round() as usize;
        gen(rng, cmp::max(age, ADULT_AGE), false)
    }

    /// Apply a childhood to the mob.
    fn train_childhood(&mut self, childhood: &Childhood) {
        // Childhood lasts for 5 years.
        self.age = 5;

        self.history.push((self.age, LifeEvent::Raised { childhood: *childhood }));

        // Every childhood gives 5 in the physicals, 2 in the mentals.
        self.agility = 5;
        self.constitution = 5;
        self.endurance = 5;
        self.recuperation = 5;
        self.strength = 5;
        self.toughness = 5;
        self.animal = 2;
        self.bargain = 2;
        self.charm = 2;
        self.chirurgy = 2;
        self.craft = 0;
        self.empathy = 2;
        self.forage = 2;
        self.guile = 2;
        self.heal = 2;
        self.hunt = 2;
        self.intuition = 2;
        self.lore = 2;

        // Then 25 are assigned as dictated by the type of childhood to 4 attributes.
        match childhood {
            &Childhood::Athletic => {
                self.agility += 10;
                self.endurance += 7;
                self.recuperation += 3;
                self.strength += 5;
            }
            &Childhood::Mischievous => {
                self.agility += 5;
                self.charm += 7;
                self.empathy += 7;
                self.guile += 10;
            }
            &Childhood::Outdoor => {
                self.animal += 3;
                self.constitution += 7;
                self.endurance += 10;
                self.hunt += 5;
            }
        }

    }

    /// Apply a training package to the mob.
    pub fn train(&mut self, package: &TrainingPackage) {
        self.age += package.years();

        self.history.push((self.age, LifeEvent::Learned { package: *package }));

        // A training package grants 15 points per year to the related attributes.
        match package {
            // Adventurer (profession): 3 years, grants 15 points in related mental attributes, 25
            // points in related competency attributes and 5 in the profession attribute.
            &TrainingPackage::Adventurer => {
                self.chirurgy += 5;
                self.heal += 5;
                self.lore += 5;
                self.competence_shield += 10;
                self.competence_single += 5;
                self.competence_sword += 10;
                self.profession_adventurer += 5;
            }
            &TrainingPackage::Mercernary => {
                self.chirurgy += 5;
                self.heal += 5;
                self.hunt += 5;
                self.competence_bow += 10;
                self.competence_great += 5;
                self.competence_spear += 10;
                self.profession_adventurer += 5;
            }

            // Adventurer (competency): 1 year, grants 5 points in related physical attributes
            // and 10 in the competency attribute.
            &TrainingPackage::BowCompetency => {
                self.strength += 5;
                self.competence_bow += 10;
            }
            &TrainingPackage::GreatCompetency => {
                self.strength += 5;
                self.competence_great += 10;
            }
            &TrainingPackage::ShieldCompetency => {
                self.strength += 3;
                self.toughness += 2;
                self.competence_shield += 10;
            }
            &TrainingPackage::SingleCompetency => {
                self.agility += 5;
                self.competence_single += 10;
            }
            &TrainingPackage::SpearCompetency => {
                self.endurance += 2;
                self.strength += 3;
                self.competence_spear += 10;
            }
            &TrainingPackage::StaffCompetency => {
                self.agility += 3;
                self.endurance += 2;
                self.competence_staff += 10;
            }
            &TrainingPackage::SwordCompetency => {
                self.agility += 2;
                self.endurance += 2;
                self.strength += 1;
                self.competence_sword += 10;
            }
            &TrainingPackage::WarhammerCompetency => {
                self.strength += 5;
                self.competence_warhammer += 10;
            }

            // Profession: lasts 3 years, grants 40 points in related non-profession attributes and
            // 5 in the profession attribute.
            &TrainingPackage::AnimalHandler => {
                self.endurance += 5;
                self.strength += 5;
                self.animal += 30;
                self.profession_animalhandler += 5;
            }
            &TrainingPackage::Apothecarist => {
                self.chirurgy += 15;
                self.forage += 5;
                self.heal += 20;
                self.profession_apothecarist += 5;
            }
            &TrainingPackage::Appraiser => {
                self.lore += 30;
                self.bargain += 10;
                self.profession_appraiser += 5;
            }
            &TrainingPackage::Cutter => {
                self.chirurgy += 25;
                self.empathy += 10;
                self.strength += 5;
                self.profession_cutter += 5;
            }
            &TrainingPackage::Farmer => {
                self.endurance += 10;
                self.strength += 10;
                self.animal += 20;
                self.profession_farmer += 5;
            }
            &TrainingPackage::Innkeeper => {
                self.bargain += 20;
                self.charm += 10;
                self.empathy += 10;
                self.profession_innkeeper += 5;
            }
            &TrainingPackage::Laborer => {
                self.endurance += 15;
                self.recuperation += 5;
                self.strength += 15;
                self.toughness += 5;
                self.profession_laborer += 5;
            }
            &TrainingPackage::Tinker => {
                self.bargain += 10;
                self.craft += 30;
                self.profession_tinker += 5;
            }
            &TrainingPackage::Trader => {
                self.bargain += 30;
                self.guile += 5;
                self.lore += 5;
                self.profession_trader += 5;
            }
            &TrainingPackage::Woodsman => {
                self.constitution += 5;
                self.endurance += 5;
                self.forage += 10;
                self.hunt += 20;
                self.profession_woodsman += 5;
            }

            // Personality (1 year)
            &TrainingPackage::Negotiation => {
                self.bargain += 5;
                self.guile += 5;
                self.charm += 5;
            }

            // Miscellaneous (1 year)
            &TrainingPackage::Athlete => {
                self.agility += 5;
                self.endurance += 5;
                self.recuperation += 5;
            }
            &TrainingPackage::Brawler => {
                self.strength += 5;
                self.toughness += 10;
            }
            &TrainingPackage::Charmer => {
                self.charm += 10;
                self.guile += 5;
            }
            &TrainingPackage::Conman => {
                self.charm += 5;
                self.guile += 10;
            }
            &TrainingPackage::Footpad => {
                self.agility += 5;
                self.charm += 5;
                self.guile += 5;
            }
            &TrainingPackage::Forager => {
                self.hunt += 5;
                self.forage += 10;
            }
        }
    }
}

/// Generate a mobile of the given age.
fn gen<R: Rng>(rng: &mut R, age: usize, is_adventurer: bool) -> Mobile {
    if age < MIN_AGE {
        panic!("Attempted to create a mob younger han {}!", MIN_AGE);
    }

    // We start off with a blank slate. This is an entirely nurture-based model of personality,
    // Mother Nature and Daddy Darwin have no part in this!
    let mut mob = Mobile {
        name: "kaffo".to_string(),
        age: 0,
        onset_age: None,
        history: vec![(0, LifeEvent::Born)],
        is_avaricious: false,
        is_brave: false,
        is_envious: false,
        is_gluttonous: false,
        is_slothful: false,
        desires: BTreeMap::new(),
        agility: 0,
        constitution: 0,
        endurance: 0,
        recuperation: 0,
        strength: 0,
        toughness: 0,
        animal: 0,
        bargain: 0,
        charm: 0,
        chirurgy: 0,
        craft: 0,
        empathy: 0,
        forage: 0,
        guile: 0,
        heal: 0,
        hunt: 0,
        intuition: 0,
        lore: 0,
        competence_bow: 0,
        competence_great: 0,
        competence_shield: 0,
        competence_single: 0,
        competence_spear: 0,
        competence_staff: 0,
        competence_sword: 0,
        competence_warhammer: 0,
        profession_adventurer: 0,
        profession_animalhandler: 0,
        profession_apothecarist: 0,
        profession_appraiser: 0,
        profession_cutter: 0,
        profession_farmer: 0,
        profession_innkeeper: 0,
        profession_laborer: 0,
        profession_tinker: 0,
        profession_trader: 0,
        profession_woodsman: 0,
    };

    // Assign personality traits randomly. Let's say that 75% of the population are not particularly
    // avaricious/brave/whatnot, and the remaining 25% are. The exception is that all adventurers
    // are brave.
    mob.is_avaricious = rng.gen_range(0, 5) == 0;
    mob.is_brave = is_adventurer || rng.gen_range(0, 5) == 0;
    mob.is_envious = rng.gen_range(0, 5) == 0;
    mob.is_gluttonous = rng.gen_range(0, 5) == 0;
    mob.is_slothful = rng.gen_range(0, 5) == 0;

    // Then, determine the age at which the mob became an adventurer. Let's say that any point after
    // MIN_ONSET is fair game.
    let (pre_years, post_years) = if is_adventurer && age > MIN_ONSET {
        let onset = rng.gen_range(MIN_ONSET, age);
        mob.onset_age = Some(onset);
        (onset, age - onset)
    } else {
        // Of course, if they're not an adventurer, or are a freshly-minted adventurer (this is
        // their first quest!), they get no years of adventurer experience.
        (age, 0)
    };

    // Now pick and apply training packages. We go in the order childhood (which always lasts
    // MIN_AGE years), pre-onset years, then post-onset years.
    mob.train_childhood(rng.choose(&CHILDHOOD).unwrap());
    random_train(rng, &mut mob, pre_years - MIN_AGE, &PRE_ONSET);

    // All adventurers start with 5 experience in the adventurer profession.
    if is_adventurer {
        mob.profession_adventurer = 5;
        mob.history.push((mob.age, LifeEvent::Onset));
        random_train(rng, &mut mob, post_years, &POST_ONSET);
    }

    mob
}

/// Randomly train a mob for a number of years.
///
/// This panics if there are no applicable training packages.
fn random_train<R: Rng>(rng: &mut R,
                        mob: &mut Mobile,
                        years: usize,
                        packages: &[TrainingPackage]) {
    // Choose the initial training package.
    let initial = {
        let mut applicable = 0;
        for package in packages {
            if package.applicable(mob) && package.years() <= years {
                applicable += 1;
            }
        }

        if applicable == 0 {
            panic!("No applicable initial training package!");
        }

        let mut idx = rng.gen_range(0, applicable);

        let mut found = None;
        for package in packages {
            if package.applicable(mob) && package.years() <= years {
                idx -= 1;
                if idx == 0 {
                    found = Some(*package);
                    break;
                }
            }
        }

        found.expect("Failed to select initial training package!")
    };

    // Train the mob.
    mob.train(&initial);

    // Then train for the remaining years.
    let mut prior = initial;
    let mut remaining = years - initial.years();
    while remaining > 0 {
        let package = choose_package(rng, mob, remaining, packages, prior);
        mob.train(&package);
        remaining -= package.years();
        prior = package;
    }
}

/// Pick a training package applicable to a mob. This is biassed towards one related to the prior,
/// as people tend not to *completely* change career, even if they theoretically could.
///
/// This assumes there are applicable training packages.
fn choose_package<R: Rng>(rng: &mut R,
                          mob: &Mobile,
                          max_years: usize,
                          packages: &[TrainingPackage],
                          prior: TrainingPackage)
                          -> TrainingPackage {
    // Determine how many packages are applicable and how many are related to the prior package.
    let mut applicable = 0;
    let mut related = 0;
    for package in packages {
        if package.applicable(mob) && package.years() <= max_years {
            applicable += 1;
            if prior.related(package) {
                related += 1;
            }
        }
    }

    // Determine what sort of package to pick: any applicable one (class 0); or any related one
    // (class 1). These classes are chosen between with equal probability. Then a package in the
    // selected class is picked uniformly. As each package is only related to a couple of others,
    // this has the effect of biassing towards related packages.
    let class = if related != 0 { rng.gen_range(0, 2) } else { 0 };

    // Pick a package in the chosen set.
    let num_options = if class == 0 { applicable } else { related };
    let mut idx = rng.gen_range(0, num_options);

    // Return the chosen package.
    for package in packages {
        if package.applicable(mob) && package.years() <= max_years {
            if class == 0 || prior.related(package) {
                if idx == 0 {
                    return *package;
                }
                idx -= 1;
            }
        }
    }

    panic!("Failed to select training package!");
}

/// Childhood training packages.
const CHILDHOOD: [Childhood; 3] = [Childhood::Athletic, Childhood::Mischievous, Childhood::Outdoor];

/// Pre-onset training packages.
const PRE_ONSET: [TrainingPackage; 17] = [TrainingPackage::AnimalHandler,
                                          TrainingPackage::Apothecarist,
                                          TrainingPackage::Appraiser,
                                          TrainingPackage::Cutter,
                                          TrainingPackage::Farmer,
                                          TrainingPackage::Innkeeper,
                                          TrainingPackage::Laborer,
                                          TrainingPackage::Tinker,
                                          TrainingPackage::Trader,
                                          TrainingPackage::Woodsman,
                                          TrainingPackage::Negotiation,
                                          TrainingPackage::Athlete,
                                          TrainingPackage::Brawler,
                                          TrainingPackage::Charmer,
                                          TrainingPackage::Conman,
                                          TrainingPackage::Footpad,
                                          TrainingPackage::Forager];

/// Post-onset training packages.
const POST_ONSET: [TrainingPackage; 23] = [TrainingPackage::Adventurer,
                                           TrainingPackage::Mercernary,
                                           TrainingPackage::BowCompetency,
                                           TrainingPackage::GreatCompetency,
                                           TrainingPackage::ShieldCompetency,
                                           TrainingPackage::SingleCompetency,
                                           TrainingPackage::SpearCompetency,
                                           TrainingPackage::StaffCompetency,
                                           TrainingPackage::SwordCompetency,
                                           TrainingPackage::WarhammerCompetency,
                                           TrainingPackage::AnimalHandler,
                                           TrainingPackage::Apothecarist,
                                           TrainingPackage::Appraiser,
                                           TrainingPackage::Tinker,
                                           TrainingPackage::Trader,
                                           TrainingPackage::Woodsman,
                                           TrainingPackage::Athlete,
                                           TrainingPackage::Brawler,
                                           TrainingPackage::Charmer,
                                           TrainingPackage::Conman,
                                           TrainingPackage::Footpad,
                                           TrainingPackage::Forager,
                                           TrainingPackage::Negotiation];

/// Types of childhoods.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Childhood {
    Athletic,
    Mischievous,
    Outdoor,
}

/// A training package. These fall into four types:
///
/// 1. Adventurer (1 and 3 year): only available to adventurers.
///
/// 2. Profession (3 year): only the animal handler, apothecarist, appraiser, tinker, trader, and
///    woodsman are available to post-onset adventurers.
///
/// 3. Personality (1 year): only available to those with the appropriate personality trait.
///
/// 4. Miscellaneous (1 year): available to all.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TrainingPackage {
    // adventurer
    Adventurer,
    Mercernary,
    BowCompetency,
    GreatCompetency,
    ShieldCompetency,
    SingleCompetency,
    SpearCompetency,
    StaffCompetency,
    SwordCompetency,
    WarhammerCompetency,

    // profession
    AnimalHandler,
    Apothecarist,
    Appraiser,
    Cutter,
    Farmer,
    Innkeeper,
    Laborer,
    Tinker,
    Trader,
    Woodsman,

    // personality
    Negotiation,

    // miscellaneous
    Athlete,
    Brawler,
    Charmer,
    Conman,
    Footpad,
    Forager,
}

impl TrainingPackage {
    /// The number of years a package takes.
    pub fn years(&self) -> usize {
        match self {
            // adventurer
            &TrainingPackage::Adventurer |
            &TrainingPackage::Mercernary => 3,

            &TrainingPackage::BowCompetency |
            &TrainingPackage::GreatCompetency |
            &TrainingPackage::ShieldCompetency |
            &TrainingPackage::SingleCompetency |
            &TrainingPackage::SpearCompetency |
            &TrainingPackage::StaffCompetency |
            &TrainingPackage::SwordCompetency |
            &TrainingPackage::WarhammerCompetency => 1,

            // profession
            &TrainingPackage::AnimalHandler |
            &TrainingPackage::Apothecarist |
            &TrainingPackage::Appraiser |
            &TrainingPackage::Cutter |
            &TrainingPackage::Farmer |
            &TrainingPackage::Innkeeper |
            &TrainingPackage::Laborer |
            &TrainingPackage::Tinker |
            &TrainingPackage::Trader |
            &TrainingPackage::Woodsman => 3,

            // personality
            &TrainingPackage::Negotiation => 1,

            // miscellaneous
            &TrainingPackage::Athlete |
            &TrainingPackage::Brawler |
            &TrainingPackage::Charmer |
            &TrainingPackage::Conman |
            &TrainingPackage::Footpad |
            &TrainingPackage::Forager => 1,
        }
    }

    /// Whether the given package is somewhat related to this one. This is a reflexive and symmetric
    /// relation, but not necessarily transitive.
    pub fn related(&self, other: &TrainingPackage) -> bool {
        *self == *other || related_non_refsym(self, other) || related_non_refsym(other, self)
    }

    /// Whether the given mob is capable of using this training package.
    pub fn applicable(&self, mob: &Mobile) -> bool {
        // Only the personality training packages are conditional.
        match self {
            &TrainingPackage::Negotiation => mob.is_avaricious || mob.is_envious,
            _ => true,
        }
    }
}

/// Non-reflexive&symmetric "related" predicate on training packages.
fn related_non_refsym(a: &TrainingPackage, b: &TrainingPackage) -> bool {
    // `aRb` iff the two packages are conceptually related, and `a` preceeds `b` in a left-hand
    // component in this list. This keeps the list of cases about half the size it would otherwise
    // be.
    match (a, b) {
        // adventurer
        (&TrainingPackage::Adventurer, &TrainingPackage::Apothecarist) => true,
        (&TrainingPackage::Adventurer, &TrainingPackage::Appraiser) => true,
        (&TrainingPackage::Adventurer, &TrainingPackage::ShieldCompetency) => true,
        (&TrainingPackage::Adventurer, &TrainingPackage::SingleCompetency) => true,
        (&TrainingPackage::Adventurer, &TrainingPackage::SwordCompetency) => true,
        (&TrainingPackage::Mercernary, &TrainingPackage::Apothecarist) => true,
        (&TrainingPackage::Mercernary, &TrainingPackage::BowCompetency) => true,
        (&TrainingPackage::Mercernary, &TrainingPackage::GreatCompetency) => true,
        (&TrainingPackage::Mercernary, &TrainingPackage::SpearCompetency) => true,
        (&TrainingPackage::Mercernary, &TrainingPackage::Woodsman) => true,
        (&TrainingPackage::GreatCompetency, &TrainingPackage::SpearCompetency) => true,
        (&TrainingPackage::GreatCompetency, &TrainingPackage::SwordCompetency) => true,
        (&TrainingPackage::GreatCompetency, &TrainingPackage::WarhammerCompetency) => true,
        (&TrainingPackage::SingleCompetency, &TrainingPackage::SwordCompetency) => true,
        (&TrainingPackage::SingleCompetency, &TrainingPackage::StaffCompetency) => true,

        // profession
        (&TrainingPackage::AnimalHandler, &TrainingPackage::Farmer) => true,
        (&TrainingPackage::Apothecarist, &TrainingPackage::Cutter) => true,
        (&TrainingPackage::Appraiser, &TrainingPackage::Tinker) => true,
        (&TrainingPackage::Appraiser, &TrainingPackage::Trader) => true,
        (&TrainingPackage::Tinker, &TrainingPackage::Trader) => true,
        (&TrainingPackage::Trader, &TrainingPackage::Charmer) => true,
        (&TrainingPackage::Trader, &TrainingPackage::Conman) => true,
        (&TrainingPackage::Woodsman, &TrainingPackage::Forager) => true,

        // miscellaneous
        (&TrainingPackage::Athlete, &TrainingPackage::Brawler) => true,
        (&TrainingPackage::Charmer, &TrainingPackage::Conman) => true,
        (&TrainingPackage::Conman, &TrainingPackage::Footpad) => true,

        // catch_all
        (_, _) => false,
    }
}
