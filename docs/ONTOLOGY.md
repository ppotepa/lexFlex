# Ontology — Concept Hierarchy and Semantic Relations

The ontology defines hierarchical relationships between concepts in the Interlingua. It enables semantic reasoning, type checking, and inference during parsing and generation.

## Concept Hierarchy

Concepts are organized in a tree structure using `IS_A` (hyponymy) and `PART_OF` (meronymy) relations.

```
PHYSICAL_OBJECT
├─ ANIMATE_ENTITY
│  ├─ PERSON
│  │  ├─ MALE_PERSON
│  │  └─ FEMALE_PERSON
│  └─ ANIMAL
│     ├─ DOMESTIC_ANIMAL
│     │  ├─ CAT
│     │  └─ DOG
│     └─ WILD_ANIMAL
│
├─ INANIMATE_OBJECT
│  ├─ FOOD
│  │  ├─ FRUIT
│  │  │  ├─ APPLE
│  │  │  └─ PEAR
│  │  ├─ VEGETABLE
│  │  └─ BEVERAGE
│  │
│  ├─ CONTAINER
│  │  ├─ CUP
│  │  └─ BOTTLE
│  │
│  ├─ TOOL
│  │  ├─ KNIFE
│  │  └─ PEN
│  │
│  ├─ ARTIFACT
│  │  ├─ BOOK
│  │  ├─ PHONE
│  │  └─ CLOTHING
│  │
│  └─ STRUCTURE
│     ├─ BUILDING
│     │  ├─ HOUSE
│     │  └─ SHOP
│     └─ FURNITURE
│        ├─ TABLE
│        └─ CHAIR
│
ABSTRACT_ENTITY
├─ EVENT
│  ├─ ACTION
│  │  ├─ TRANSFER_ACTION
│  │  │  ├─ GIVE
│  │  │  ├─ TAKE
│  │  │  └─ SEND
│  │  ├─ MOTION_ACTION
│  │  │  ├─ GO
│  │  │  ├─ COME
│  │  │  └─ RUN
│  │  └─ CREATION_ACTION
│  │     ├─ MAKE
│  │     └─ BUILD
│  │
│  └─ MENTAL_EVENT
│     ├─ PERCEPTION
│     │  ├─ SEE
│     │  └─ HEAR
│     ├─ COGNITION
│     │  ├─ THINK
│     │  ├─ KNOW
│     │  └─ BELIEVE
│     └─ EMOTION
│        ├─ LOVE
│        ├─ HATE
│        └─ FEAR
│
├─ PROPERTY
│  ├─ PHYSICAL_PROPERTY
│  │  ├─ SIZE
│  │  │  ├─ BIG
│  │  │  └─ SMALL
│  │  ├─ COLOR
│  │  │  ├─ RED
│  │  │  ├─ BLUE
│  │  │  └─ GREEN
│  │  └─ SHAPE
│  │     ├─ ROUND
│  │     └─ SQUARE
│  │
│  └─ EVALUATIVE_PROPERTY
│     ├─ GOOD
│     ├─ BAD
│     ├─ BEAUTIFUL
│     └─ UGLY
│
├─ QUANTITY
│  ├─ NUMBER
│  └─ AMOUNT
│
└─ TIME
   ├─ MOMENT
   │  ├─ NOW
   │  ├─ TODAY
   │  └─ YESTERDAY
   └─ DURATION
```

## Ontology Data Structure

```rust
pub struct Ontology {
    /// Map from concept ID to concept definition
    pub concepts: HashMap<ConceptId, ConceptDefinition>,
    
    /// IS_A relations (child → parent)
    pub is_a: HashMap<ConceptId, Vec<ConceptId>>,
    
    /// PART_OF relations (part → whole)
    pub part_of: HashMap<ConceptId, Vec<ConceptId>>,
    
    /// Semantic type constraints
    pub type_constraints: Vec<TypeConstraint>,
}

pub struct ConceptDefinition {
    pub id: ConceptId,
    pub name: String,
    pub category: ConceptCategory,
    pub inherent_features: FeatureBundle,
    
    /// What frame this concept can participate in
    pub allowed_frames: Vec<FrameType>,
    
    /// What roles this concept can fill
    pub allowed_roles: Vec<SemanticRole>,
    
    /// Ontological properties
    pub properties: ConceptProperties,
}

pub struct ConceptProperties {
    /// Can this concept be counted? (apple: yes, water: no)
    pub countability: Countability,
    
    /// Is this concept concrete or abstract?
    pub concreteness: Concreteness,
    
    /// Typical size (for physical objects)
    pub typical_size: Option<Size>,
    
    /// Typical color (if characteristic)
    pub typical_color: Option<Color>,
}
```

## Type Constraints

Type constraints define what kinds of entities can fill specific roles in frames. This enables semantic validation during parsing.

```rust
pub struct TypeConstraint {
    /// Frame type this constraint applies to
    pub frame: FrameType,
    
    /// Role within the frame
    pub role: SemanticRole,
    
    /// Required semantic type (concept must be descendant of this)
    pub required_type: ConceptId,
    
    /// Optional: forbidden types (concept must NOT be descendant of this)
    pub forbidden_types: Vec<ConceptId>,
}
```

### Example Constraints

```rust
// GIVE frame constraints
TypeConstraint {
    frame: FrameType::Transfer,
    role: SemanticRole::Agent,
    required_type: ANIMATE_ENTITY,  // agent must be animate
    forbidden_types: vec![],
}

TypeConstraint {
    frame: FrameType::Transfer,
    role: SemanticRole::Theme,
    required_type: PHYSICAL_OBJECT,  // theme must be physical
    forbidden_types: vec![ABSTRACT_ENTITY],
}

// EAT frame constraints
TypeConstraint {
    frame: FrameType::Destruction,
    role: SemanticRole::Agent,
    required_type: ANIMATE_ENTITY,  // eater must be animate
    forbidden_types: vec![],
}

TypeConstraint {
    frame: FrameType::Destruction,
    role: SemanticRole::Theme,
    required_type: FOOD,  // eaten thing must be food
    forbidden_types: vec![],
}

// SEE frame constraints
TypeConstraint {
    frame: FrameType::Perception,
    role: SemanticRole::Agent,
    required_type: PERSON,  // seer must be person (has eyes)
    forbidden_types: vec![INANIMATE_OBJECT],
}

TypeConstraint {
    frame: FrameType::Perception,
    role: SemanticRole::Theme,
    required_type: PHYSICAL_OBJECT,  // seen thing must be physical
    forbidden_types: vec![ABSTRACT_ENTITY],
}
```

## Type Checking During Parsing

When parsing a sentence, the engine validates that entities match the type constraints for their roles:

```rust
pub fn validate_semantic_types(
    frame: &Frame,
    ontology: &Ontology,
) -> Result<(), SemanticError> {
    let frame_type = frame.frame_type();
    
    for (role, entity) in frame.roles.iter() {
        // Find applicable constraints
        let constraints = ontology.constraints_for(frame_type, *role);
        
        for constraint in constraints {
            let entity_concept = entity.concept;
            
            // Check required type
            if !ontology.is_descendant_of(entity_concept, constraint.required_type) {
                return Err(SemanticError::TypeMismatch {
                    role: *role,
                    expected: constraint.required_type,
                    found: entity_concept,
                });
            }
            
            // Check forbidden types
            for forbidden in &constraint.forbidden_types {
                if ontology.is_descendant_of(entity_concept, *forbidden) {
                    return Err(SemanticError::ForbiddenType {
                        role: *role,
                        forbidden: *forbidden,
                        found: entity_concept,
                    });
                }
            }
        }
    }
    
    Ok(())
}
```

### Example: Semantic Validation

```
Input: "Tomek zjadł jabłko."
       (Tomek ate an apple.)

Parsing:
  - Frame: Destruction (EAT)
  - Agent: Tomek (PERSON)
  - Theme: jabłko (APPLE)

Validation:
  - Constraint: EAT agent must be ANIMATE_ENTITY
    ✓ PERSON IS_A ANIMATE_ENTITY
    
  - Constraint: EAT theme must be FOOD
    ✓ APPLE IS_A FRUIT IS_A FOOD
    
  Result: VALID

---

Input: "Kamień zjadł jabłko."
       (The stone ate an apple.)

Parsing:
  - Frame: Destruction (EAT)
  - Agent: kamień (STONE → PHYSICAL_OBJECT)
  - Theme: jabłko (APPLE)

Validation:
  - Constraint: EAT agent must be ANIMATE_ENTITY
    ✗ PHYSICAL_OBJECT is NOT descendant of ANIMATE_ENTITY
      (STONE IS_A PHYSICAL_OBJECT IS_A INANIMATE_OBJECT)
    
  Result: SemanticError::TypeMismatch
          (stones cannot eat — semantic anomaly)
```

## Inheritance of Features

Concepts inherit features from their ancestors in the ontology:

```rust
impl Ontology {
    pub fn get_inherited_features(&self, concept: ConceptId) -> FeatureBundle {
        let mut features = FeatureBundle::default();
        
        // Collect features from all ancestors (bottom-up)
        let ancestors = self.get_ancestors(concept);
        
        for ancestor in ancestors.iter().rev() {
            let ancestor_def = self.concepts.get(ancestor).unwrap();
            features.merge_with(&ancestor_def.inherent_features);
        }
        
        // Override with concept's own features
        let concept_def = self.concepts.get(&concept).unwrap();
        features.merge_with(&concept_def.inherent_features);
        
        features
    }
}
```

### Example: Feature Inheritance

```
APPLE:
  - Own features: {gender: neuter, countability: count}

FRUIT (parent):
  - Inherent features: {concreteness: concrete}

FOOD (grandparent):
  - Inherent features: {edibility: edible}

PHYSICAL_OBJECT (great-grandparent):
  - Inherent features: {tangibility: tangible}

Inherited features for APPLE:
  {
    tangibility: tangible,      // from PHYSICAL_OBJECT
    edibility: edible,          // from FOOD
    concreteness: concrete,     // from FRUIT
    gender: neuter,             // own
    countability: count,        // own
  }
```

## IS_A Reasoning

The `IS_A` relation enables semantic inference:

```rust
impl Ontology {
    /// Check if concept_a IS_A concept_b
    /// (concept_a is a hyponym of concept_b)
    pub fn is_a(&self, concept_a: ConceptId, concept_b: ConceptId) -> bool {
        if concept_a == concept_b {
            return true;
        }
        
        // Check direct parents
        if let Some(parents) = self.is_a.get(&concept_a) {
            for parent in parents {
                if *parent == concept_b {
                    return true;
                }
                
                // Recursively check ancestors
                if self.is_a(*parent, concept_b) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Check if concept_a is a descendant of concept_b
    /// (concept_b is an ancestor of concept_a)
    pub fn is_descendant_of(&self, concept_a: ConceptId, concept_b: ConceptId) -> bool {
        self.is_a(concept_a, concept_b)
    }
    
    /// Get all ancestors of a concept
    pub fn get_ancestors(&self, concept: ConceptId) -> Vec<ConceptId> {
        let mut ancestors = Vec::new();
        let mut current = concept;
        
        while let Some(parents) = self.is_a.get(&current) {
            if let Some(parent) = parents.first() {
                ancestors.push(*parent);
                current = *parent;
            } else {
                break;
            }
        }
        
        ancestors
    }
}
```

## PART_OF Reasoning

The `PART_OF` relation (meronymy) describes composition:

```rust
pub struct PartOfRelation {
    pub part: ConceptId,
    pub whole: ConceptId,
    pub relation_type: PartOfType,
}

pub enum PartOfType {
    /// Physical component (wheel PART_OF car)
    Component,
    
    /// Member of group (tree PART_OF forest)
    Member,
    
    /// Portion of substance (slice PART_OF cake)
    Portion,
}
```

### Example PART_OF Relations

```
Component:
  - WHEEL PART_OF CAR
  - PAGE PART_OF BOOK
  - HANDLE PART_OF DOOR

Member:
  - TREE PART_OF FOREST
  - PLAYER PART_OF TEAM
  - WORD PART_OF SENTENCE

Portion:
  - SLICE PART_OF CAKE
  - DROP PART_OF WATER
  - PIECE PART_OF BREAD
```

## Semantic Roles and Ontology

The ontology constrains which concepts can fill which semantic roles:

```rust
impl ConceptDefinition {
    /// Check if this concept can fill a given role
    pub fn can_fill_role(&self, role: SemanticRole) -> bool {
        self.allowed_roles.contains(&role)
    }
    
    /// Check if this concept can participate in a given frame
    pub fn can_participate_in(&self, frame: FrameType) -> bool {
        self.allowed_frames.contains(&frame)
    }
}
```

### Role Constraints by Concept Category

```
ANIMATE_ENTITY:
  - Can fill: Agent, Experiencer, Recipient, Beneficiary
  - Cannot fill: Theme (usually), Location, Instrument

PHYSICAL_OBJECT:
  - Can fill: Theme, Patient, Location, Instrument
  - Cannot fill: Agent (unless personified)

PERSON:
  - Can fill: Agent, Experiencer, Recipient, Beneficiary, Theme
  - Most flexible role assignment

ABSTRACT_ENTITY:
  - Can fill: Theme (in cognitive frames), Stimulus
  - Cannot fill: Agent, Location, Instrument
```

## Ontology in RON

The ontology is stored in `data/ontology/ontology.ron`:

```ron
Ontology(
    concepts: {
        PHYSICAL_OBJECT: ConceptDefinition(
            id: "PHYSICAL_OBJECT",
            name: "Physical Object",
            category: Entity,
            inherent_features: FeatureBundle(
                concreteness: Some(Concrete),
                tangibility: Some(Tangible),
            ),
            allowed_frames: [],
            allowed_roles: [Theme, Patient, Location, Instrument],
            properties: ConceptProperties(
                countability: Both,
                concreteness: Concrete,
                typical_size: None,
                typical_color: None,
            ),
        ),
        
        ANIMATE_ENTITY: ConceptDefinition(
            id: "ANIMATE_ENTITY",
            name: "Animate Entity",
            category: Entity,
            inherent_features: FeatureBundle(
                animacy: Some(Animate),
            ),
            allowed_frames: [],
            allowed_roles: [Agent, Experiencer, Recipient, Beneficiary],
            properties: ConceptProperties(
                countability: Count,
                concreteness: Concrete,
                typical_size: None,
                typical_color: None,
            ),
        ),
        
        PERSON: ConceptDefinition(
            id: "PERSON",
            name: "Person",
            category: Entity,
            inherent_features: FeatureBundle(
                person: Some(Third),
                animacy: Some(Animate),
                concreteness: Some(Concrete),
            ),
            allowed_frames: [Transfer, Motion, Perception, Cognition, Emotion, Communication],
            allowed_roles: [Agent, Experiencer, Recipient, Beneficiary, Theme],
            properties: ConceptProperties(
                countability: Count,
                concreteness: Concrete,
                typical_size: None,
                typical_color: None,
            ),
        ),
        
        FOOD: ConceptDefinition(
            id: "FOOD",
            name: "Food",
            category: Entity,
            inherent_features: FeatureBundle(
                edibility: Some(Edible),
                concreteness: Some(Concrete),
            ),
            allowed_frames: [Destruction],
            allowed_roles: [Theme, Patient],
            properties: ConceptProperties(
                countability: Both,
                concreteness: Concrete,
                typical_size: Some(Small),
                typical_color: None,
            ),
        ),
        
        APPLE: ConceptDefinition(
            id: "APPLE",
            name: "Apple",
            category: Entity,
            inherent_features: FeatureBundle(
                gender: Some(Neuter),  // in Polish
                countability: Some(Count),
                concreteness: Some(Concrete),
                typical_color: Some(Red),
            ),
            allowed_frames: [Transfer, Destruction],
            allowed_roles: [Theme, Patient],
            properties: ConceptProperties(
                countability: Count,
                concreteness: Concrete,
                typical_size: Some(Small),
                typical_color: Some(Red),
            ),
        ),
        
        // ... more concepts
    },
    
    is_a: {
        // child → [parents]
        "ANIMATE_ENTITY": ["PHYSICAL_OBJECT"],
        "PERSON": ["ANIMATE_ENTITY"],
        "MALE_PERSON": ["PERSON"],
        "FEMALE_PERSON": ["PERSON"],
        "ANIMAL": ["ANIMATE_ENTITY"],
        "DOMESTIC_ANIMAL": ["ANIMAL"],
        "CAT": ["DOMESTIC_ANIMAL"],
        "DOG": ["DOMESTIC_ANIMAL"],
        "INANIMATE_OBJECT": ["PHYSICAL_OBJECT"],
        "FOOD": ["INANIMATE_OBJECT"],
        "FRUIT": ["FOOD"],
        "APPLE": ["FRUIT"],
        "PEAR": ["FRUIT"],
        "VEGETABLE": ["FOOD"],
        "BEVERAGE": ["FOOD"],
        "CONTAINER": ["INANIMATE_OBJECT"],
        "CUP": ["CONTAINER"],
        "BOTTLE": ["CONTAINER"],
        "TOOL": ["INANIMATE_OBJECT"],
        "KNIFE": ["TOOL"],
        "PEN": ["TOOL"],
        "ARTIFACT": ["INANIMATE_OBJECT"],
        "BOOK": ["ARTIFACT"],
        "PHONE": ["ARTIFACT"],
        "CLOTHING": ["ARTIFACT"],
        "STRUCTURE": ["INANIMATE_OBJECT"],
        "BUILDING": ["STRUCTURE"],
        "HOUSE": ["BUILDING"],
        "SHOP": ["BUILDING"],
        "FURNITURE": ["STRUCTURE"],
        "TABLE": ["FURNITURE"],
        "CHAIR": ["FURNITURE"],
        
        "ABSTRACT_ENTITY": [],
        "EVENT": ["ABSTRACT_ENTITY"],
        "ACTION": ["EVENT"],
        "TRANSFER_ACTION": ["ACTION"],
        "GIVE": ["TRANSFER_ACTION"],
        "TAKE": ["TRANSFER_ACTION"],
        "SEND": ["TRANSFER_ACTION"],
        "MOTION_ACTION": ["ACTION"],
        "GO": ["MOTION_ACTION"],
        "COME": ["MOTION_ACTION"],
        "RUN": ["MOTION_ACTION"],
        "CREATION_ACTION": ["ACTION"],
        "MAKE": ["CREATION_ACTION"],
        "BUILD": ["CREATION_ACTION"],
        "MENTAL_EVENT": ["EVENT"],
        "PERCEPTION": ["MENTAL_EVENT"],
        "SEE": ["PERCEPTION"],
        "HEAR": ["PERCEPTION"],
        "COGNITION": ["MENTAL_EVENT"],
        "THINK": ["COGNITION"],
        "KNOW": ["COGNITION"],
        "BELIEVE": ["COGNITION"],
        "EMOTION": ["MENTAL_EVENT"],
        "LOVE": ["EMOTION"],
        "HATE": ["EMOTION"],
        "FEAR": ["EMOTION"],
        
        "PROPERTY": ["ABSTRACT_ENTITY"],
        "PHYSICAL_PROPERTY": ["PROPERTY"],
        "SIZE": ["PHYSICAL_PROPERTY"],
        "BIG": ["SIZE"],
        "SMALL": ["SIZE"],
        "COLOR": ["PHYSICAL_PROPERTY"],
        "RED": ["COLOR"],
        "BLUE": ["COLOR"],
        "GREEN": ["COLOR"],
        "SHAPE": ["PHYSICAL_PROPERTY"],
        "ROUND": ["SHAPE"],
        "SQUARE": ["SHAPE"],
        "EVALUATIVE_PROPERTY": ["PROPERTY"],
        "GOOD": ["EVALUATIVE_PROPERTY"],
        "BAD": ["EVALUATIVE_PROPERTY"],
        "BEAUTIFUL": ["EVALUATIVE_PROPERTY"],
        "UGLY": ["EVALUATIVE_PROPERTY"],
        
        "QUANTITY": ["ABSTRACT_ENTITY"],
        "NUMBER": ["QUANTITY"],
        "AMOUNT": ["QUANTITY"],
        "TIME": ["ABSTRACT_ENTITY"],
        "MOMENT": ["TIME"],
        "NOW": ["MOMENT"],
        "TODAY": ["MOMENT"],
        "YESTERDAY": ["MOMENT"],
        "DURATION": ["TIME"],
    },
    
    part_of: {
        "WHEEL": [PartOfRelation(part: "WHEEL", whole: "CAR", relation_type: Component)],
        "PAGE": [PartOfRelation(part: "PAGE", whole: "BOOK", relation_type: Component)],
        "HANDLE": [PartOfRelation(part: "HANDLE", whole: "DOOR", relation_type: Component)],
        "TREE": [PartOfRelation(part: "TREE", whole: "FOREST", relation_type: Member)],
        "PLAYER": [PartOfRelation(part: "PLAYER", whole: "TEAM", relation_type: Member)],
        "WORD": [PartOfRelation(part: "WORD", whole: "SENTENCE", relation_type: Member)],
    },
    
    type_constraints: [
        // Transfer frame (GIVE, TAKE, SEND)
        TypeConstraint(frame: Transfer, role: Agent, required_type: "ANIMATE_ENTITY", forbidden_types: []),
        TypeConstraint(frame: Transfer, role: Theme, required_type: "PHYSICAL_OBJECT", forbidden_types: ["ABSTRACT_ENTITY"]),
        TypeConstraint(frame: Transfer, role: Recipient, required_type: "ANIMATE_ENTITY", forbidden_types: []),
        
        // Motion frame (GO, COME, RUN)
        TypeConstraint(frame: Motion, role: Agent, required_type: "ANIMATE_ENTITY", forbidden_types: []),
        TypeConstraint(frame: Motion, role: Source, required_type: "PHYSICAL_OBJECT", forbidden_types: []),
        TypeConstraint(frame: Motion, role: Goal, required_type: "PHYSICAL_OBJECT", forbidden_types: []),
        
        // Destruction frame (EAT, DRINK, BREAK)
        TypeConstraint(frame: Destruction, role: Agent, required_type: "ANIMATE_ENTITY", forbidden_types: []),
        TypeConstraint(frame: Destruction, role: Theme, required_type: "PHYSICAL_OBJECT", forbidden_types: []),
        
        // Perception frame (SEE, HEAR)
        TypeConstraint(frame: Perception, role: Agent, required_type: "PERSON", forbidden_types: ["INANIMATE_OBJECT"]),
        TypeConstraint(frame: Perception, role: Theme, required_type: "PHYSICAL_OBJECT", forbidden_types: []),
        
        // Cognition frame (THINK, KNOW, BELIEVE)
        TypeConstraint(frame: Cognition, role: Agent, required_type: "PERSON", forbidden_types: []),
        TypeConstraint(frame: Cognition, role: Theme, required_type: "ABSTRACT_ENTITY", forbidden_types: []),
        
        // Emotion frame (LOVE, HATE, FEAR)
        TypeConstraint(frame: Emotion, role: Agent, required_type: "ANIMATE_ENTITY", forbidden_types: []),
        TypeConstraint(frame: Emotion, role: Theme, required_type: "PHYSICAL_OBJECT", forbidden_types: []),
        
        // Communication frame (SAY, TELL, ASK)
        TypeConstraint(frame: Communication, role: Agent, required_type: "PERSON", forbidden_types: []),
        TypeConstraint(frame: Communication, role: Theme, required_type: "ABSTRACT_ENTITY", forbidden_types: []),
    ],
)
```

## Ontology Usage Examples

### 1. Semantic Validation

```rust
// Parse: "Kamień zjadł jabłko." (The stone ate an apple.)

let frame = Frame::Destruction {
    agent: Entity { concept: STONE, .. },
    theme: Entity { concept: APPLE, .. },
};

// Validate
let result = ontology.validate_semantic_types(&frame);

// Result: Err(SemanticError::TypeMismatch {
//   role: Agent,
//   expected: ANIMATE_ENTITY,
//   found: STONE  // STONE is INANIMATE, not ANIMATE
// })
```

### 2. Concept Generalization

```rust
// "Tomek dał jabłko Izie."
// "Tomek dał gruszkę Izie."
// "Tomek dał chleb Izie."

// All three themes are FOOD
// Can generalize: "Tomek dał jedzenie Izie." (Tomek gave food to Iza.)

let apple = ConceptId::APPLE;
let pear = ConceptId::PEAR;
let bread = ConceptId::BREAD;

let common_ancestor = ontology.find_common_ancestor(&[apple, pear, bread]);
// Result: Some(FOOD)
```

### 3. Semantic Similarity

```rust
// Calculate semantic distance between concepts
pub fn semantic_distance(&self, a: ConceptId, b: ConceptId) -> usize {
    let ancestors_a = self.get_ancestors(a);
    let ancestors_b = self.get_ancestors(b);
    
    // Find lowest common ancestor
    let lca = ancestors_a.iter()
        .find(|anc| ancestors_b.contains(anc))
        .copied();
    
    if let Some(lca) = lca {
        let dist_a = ancestors_a.iter().position(|x| *x == lca).unwrap_or(0);
        let dist_b = ancestors_b.iter().position(|x| *x == lca).unwrap_or(0);
        dist_a + dist_b
    } else {
        usize::MAX  // no common ancestor
    }
}

// distance(APPLE, PEAR) = 2 (APPLE→FRUIT←PEAR)
// distance(APPLE, DOG) = 6 (APPLE→FRUIT→FOOD→INANIMATE←ANIMATE←DOG)
```

## Ontology Maintenance

The ontology grows as new concepts are added to the lexicon:

```rust
impl Ontology {
    /// Add a new concept to the ontology
    pub fn add_concept(
        &mut self,
        concept: ConceptDefinition,
        parent: Option<ConceptId>,
    ) {
        let id = concept.id.clone();
        self.concepts.insert(id.clone(), concept);
        
        if let Some(parent_id) = parent {
            self.is_a.entry(id).or_insert_with(Vec::new).push(parent_id);
        }
    }
    
    /// Validate ontology consistency
    pub fn validate(&self) -> Result<(), OntologyError> {
        // Check for cycles in IS_A
        for concept in self.concepts.keys() {
            if self.has_cycle(*concept) {
                return Err(OntologyError::CycleDetected(*concept));
            }
        }
        
        // Check that all IS_A parents exist
        for (child, parents) in &self.is_a {
            for parent in parents {
                if !self.concepts.contains_key(parent) {
                    return Err(OntologyError::MissingConcept(*parent));
                }
            }
        }
        
        Ok(())
    }
}
```

## Summary

The ontology provides:

1. **Hierarchical organization** of concepts using IS_A and PART_OF relations
2. **Semantic type checking** to validate that entities match role constraints
3. **Feature inheritance** so concepts automatically inherit properties from ancestors
4. **Semantic reasoning** for inference and generalization
5. **Anomaly detection** to catch semantic errors like "stones eat apples"

The ontology is stored in `data/ontology/ontology.ron` and loaded at startup for use during parsing and generation.
