#!/bin/bash
#
# lexFlex Benchmark Generator (shell version)
#
# Generates ~500+ sentences to maximally exhaust current capabilities (see docs/UNIFIED... for pipeline target).
# Covers (algorithmic where possible via current morph/lex):
# - adjectives + nouns + degree hints (planned)
# - various inflections (verb gender, aspect pairs, noun cases via syntax)
# - enumerations / lists with "i" / "and"
# - negation, questions, temporals, quantification (incl. Numerical)
# - singular/plural hints + case after numbers (genitive in PL)
# - multiple frames + existence + possession
# - both directions PL->EN / EN->PL
#
# Usage:
#   ./scripts/generate_benchmark.sh                    # writes to results/benchmarks/benchmark_sentences.txt
#   ./scripts/generate_benchmark.sh benchmarks/my.txt  # explicit file
#   ./scripts/generate_benchmark.sh | head -100        # pipe mode (no default file)
#
# All words and structures come from existing data/lexicons and morphology.
# When extending pipeline, update this header + all docs for two-way consistency.
# Iterate: add cases → test → analyze gaps in parser/generation → refine.

set -euo pipefail

OUTFILE="${1:-}"

# Default to results/ for all generated benchmark data
if [[ -z "$OUTFILE" ]]; then
    mkdir -p "results/benchmarks"
    OUTFILE="results/benchmarks/benchmark_sentences.txt"
    echo "No output file specified — defaulting to $OUTFILE" >&2
fi

# ----------------------------- PL vocabulary -----------------------------
PL_PERSONS=(Tomek Iza Mama Tata Student Profesor)
PL_ANIMALS=(Kot Pies)
PL_THINGS=(jabłko książkę chleb mleko wodę dom okno drzewo samochód miasto)
PL_ALL_NOUNS=("${PL_PERSONS[@]}" "${PL_ANIMALS[@]}" "${PL_THINGS[@]}")

PL_ADJS=(duży mały dobry zły czerwony niebieski gorący zimny)

# Common surface verb forms that parser accepts (from lexicon + working examples)
PL_TRANSFER_VERBS=(dał dała)
PL_PERCEPTION_VERBS=(widział widziała czytał czytała)
PL_CONSUMPTION_VERBS=(jadł jadła pił piła)
PL_EMOTION_VERBS=(kochał kochała)
PL_CREATION_VERBS=(zrobił zrobiła)
PL_MOTION_VERBS=(poszedł poszła)
PL_COGNITION_VERBS=(myślał myślała)

PL_TEMPORALS=(wczoraj dzisiaj jutro)

PL_QUANTS=(Wszyscy Nikt Niektórzy Kilka Wiele)

# Aspect / perfective variants that exist in data
PL_ASPECT_VARIANTS=(
    "zjadł jabłko"
    "wypił mleko"
    "zrobił dom"
)

# ----------------------------- EN vocabulary -----------------------------
EN_PERSONS=(Tom Iza Mary "a student" "a professor" "a mother" "a father")
EN_ANIMALS=("a cat" "a dog")
EN_THINGS=("an apple" "a book" bread milk water "a house" "a window" "a tree" "a car" "a city")
EN_ALL_NOUNS=("${EN_PERSONS[@]}" "${EN_ANIMALS[@]}" "${EN_THINGS[@]}")

EN_ADJS=(big small good bad red blue hot cold)

EN_TRANSFER_VERBS=(gave give)
EN_PERCEPTION_VERBS=(saw see read)
EN_CONSUMPTION_VERBS=(ate eat drank drink)
EN_EMOTION_VERBS=(loved love)
EN_CREATION_VERBS=(made make)
EN_MOTION_VERBS=(went go)
EN_COGNITION_VERBS=(thought think)

EN_TEMPORALS=(yesterday today tomorrow)

EN_QUANTS=(All None Some Many Few)

# ----------------------------- Helpers -----------------------------
rand_choice() {
    local arr=("$@")
    echo "${arr[RANDOM % ${#arr[@]}]}"
}

make_list_pl() {
    # Simple enumerations
    local a b c
    a=$(rand_choice "${PL_ALL_NOUNS[@]}")
    b=$(rand_choice "${PL_ALL_NOUNS[@]}")
    while [[ "$b" == "$a" ]]; do b=$(rand_choice "${PL_ALL_NOUNS[@]}"); done
    echo "$a i $b"
}

make_list_en() {
    local a b
    a=$(rand_choice "${EN_ALL_NOUNS[@]}")
    b=$(rand_choice "${EN_ALL_NOUNS[@]}")
    while [[ "$b" == "$a" ]]; do b=$(rand_choice "${EN_ALL_NOUNS[@]}"); done
    echo "$a and $b"
}

# ----------------------------- Generators -----------------------------
emit() {
    echo "$1"
}

generate_pl_basic() {
    # Massive combinatorial generation for volume + coverage

    # 1. Core Transfer + adjectives + recipients
    for subj in "${PL_PERSONS[@]}"; do
        for obj in jabłko książkę chleb mleko wodę; do
            for v in "${PL_TRANSFER_VERBS[@]}"; do
                emit "PL->EN $subj $v $obj Izie"
                # with adjective on object
                for adj in "${PL_ADJS[@]:0:5}"; do
                    emit "PL->EN $subj $v ${adj} $obj Izie"
                done
            done
        done
    done

    # 2. Perception + adjectives on subject or object
    for subj in "${PL_PERSONS[@]}" "${PL_ANIMALS[@]}"; do
        for obj in jabłko książkę kota psa; do
            for v in "${PL_PERCEPTION_VERBS[@]}"; do
                emit "PL->EN $subj $v $obj"
                for adj in "${PL_ADJS[@]:0:4}"; do
                    emit "PL->EN $subj $v ${adj} $obj"
                    emit "PL->EN ${adj} $subj $v $obj"
                done
            done
        done
    done

    # 3. Consumption + aspect variants
    for subj in Tomek Iza Kot Pies; do
        for obj in jabłko chleb mleko wodę; do
            emit "PL->EN $subj jadł $obj"
            emit "PL->EN $subj zjadł $obj"
            emit "PL->EN $subj pił $obj"
            emit "PL->EN $subj wypił $obj"
        done
    done

    # 4. Emotion / Creation / Motion / Cognition
    for subj in "${PL_PERSONS[@]:0:5}"; do
        emit "PL->EN $subj kochał Izę"
        emit "PL->EN $subj kochał kota"
        emit "PL->EN $subj zrobił dom"
        emit "PL->EN $subj zrobił książkę"
        emit "PL->EN $subj poszedł do miasta"
        emit "PL->EN $subj myślał o jabłku"
    done

    # 5. Gender variants (dał/dała etc.)
    for subj in Iza Mama Tata; do
        emit "PL->EN $subj dała jabłko Tomkowi"
        emit "PL->EN $subj widziała kota"
    done

    # 6. Negation (with genitive-like forms where data has them)
    for subj in Tomek Iza Student Profesor; do
        emit "PL->EN $subj nie dał jabłka Izie"
        emit "PL->EN $subj nie widział książki"
        emit "PL->EN $subj nie jadł chleba"
        emit "PL->EN $subj nie pił mleka"
        emit "PL->EN $subj nie kochał kota"
    done

    # 7. Questions
    for subj in Tomek Iza Kot Profesor; do
        emit "PL->EN Czy $subj dał jabłko Izie"
        emit "PL->EN Czy $subj widział dużego kota"
        emit "PL->EN Czy $subj jadł chleb"
        emit "PL->EN Czy $subj poszedł wczoraj"
    done

    # 8. Temporals in different positions
    for t in "${PL_TEMPORALS[@]}"; do
        emit "PL->EN Tomek dał jabłko Izie $t"
        emit "PL->EN Iza widziała kota $t"
        emit "PL->EN $t Tomek poszedł do miasta"
    done

    # 9. Quantification
    for q in "${PL_QUANTS[@]}"; do
        emit "PL->EN $q widzieli kota"
        emit "PL->EN $q jadł jabłko"
        emit "PL->EN $q pił mleko"
    done

    # 10. Enumerations and lists (exhaust coordination)
    emit "PL->EN Tomek i Iza dał jabłko"
    emit "PL->EN Kot i Pies poszli do miasta"
    emit "PL->EN Tomek dał jabłko, książkę i chleb"
    emit "PL->EN Duży kot i mały pies pili wodę"
    emit "PL->EN Mama i Tata zrobili dom"

    # 11. Existence + Statement with adj
    for adj in "${PL_ADJS[@]}"; do
        emit "PL->EN Jest ${adj} dom"
        emit "PL->EN Jest ${adj} kot"
    done
    emit "PL->EN Duży czerwony jabłko jest na stole"
}

generate_pl_enhanced() {
    # Extra adjective combos + aspect + lists
    for adj1 in duży mały dobry; do
        for adj2 in czerwony niebieski; do
            emit "PL->EN ${adj1} ${adj2} jabłko jest w domu"
            emit "PL->EN Tomek widział ${adj1} ${adj2} książkę"
        done
    done

    emit "PL->EN Tomek nie zjadł jabłka wczoraj"
    emit "PL->EN Iza nie wypiła mleka dzisiaj"
    emit "PL->EN Czy Profesor nie poszedł jutro"

    emit "PL->EN Mama i Tata kupili dom"
    emit "PL->EN Student dał jabłko i książkę"
    emit "PL->EN Tomek dał wodę profesorowi"
    emit "PL->EN Iza dała chleb mamie"

    # More aspect + gender
    emit "PL->EN Duża Iza zjadła jabłko"
    emit "PL->EN Mali studenci poszli do miasta"
}

generate_en_basic() {
    # Heavy combinatorial for EN->PL

    # Transfer + articles + adjs
    for subj in "${EN_PERSONS[@]:0:6}"; do
        for obj in "an apple" "a book" bread milk water; do
            emit "EN->PL $subj gave $obj to Iza"
            for adj in "${EN_ADJS[@]:0:5}"; do
                emit "EN->PL $subj gave $adj $obj to Mary"
            done
        done
    done

    # Perception + Consumption
    for subj in Tom Iza "a student" "a cat"; do
        for obj in "an apple" "a book" "a cat" bread; do
            emit "EN->PL $subj saw $obj"
            emit "EN->PL $subj ate $obj"
            emit "EN->PL $subj drank milk"
        done
    done

    # Adjectives everywhere
    for adj in "${EN_ADJS[@]}"; do
        emit "EN->PL $adj cat saw a dog"
        emit "EN->PL Tom ate $adj apple yesterday"
        emit "EN->PL A $adj student read the book"
    done

    # Negation + questions + do support
    for subj in Tom Iza "a professor" "a mother"; do
        emit "EN->PL $subj did not give an apple to Mary"
        emit "EN->PL $subj did not see the big house"
        emit "EN->PL Did $subj give an apple to Iza?"
        emit "EN->PL Did $subj see a cat?"
    done

    # Temporals
    for t in "${EN_TEMPORALS[@]}"; do
        emit "EN->PL Tom went to the city $t"
        emit "EN->PL Iza saw a big dog $t"
    done

    # Quant + lists
    for q in "${EN_QUANTS[@]}"; do
        emit "EN->PL $q students ate bread"
        emit "EN->PL $q saw a red car"
    done

    # Enumerations
    emit "EN->PL Tom and Iza gave an apple to Mary"
    emit "EN->PL A big cat and a small dog went to the city"
    emit "EN->PL Tom gave an apple, a book and bread to Iza"
    emit "EN->PL Many good students and teachers read the book"

    # Existence / copula
    for adj in "${EN_ADJS[@]:0:4}"; do
        emit "EN->PL An apple is $adj"
        emit "EN->PL The $adj house is in the city"
    done
}

generate_en_enhanced() {
    # More complex NPs
    for adj in big small good; do
        emit "EN->PL The $adj red book is on the table"
        emit "EN->PL Many $adj students saw a teacher"
    done

    # Aspect-ish and plurals
    emit "EN->PL The students read many books yesterday"
    emit "EN->PL Tom drank hot milk today"

    # Neg + question + temporal
    emit "EN->PL Did Iza not love Tom yesterday?"

    # Lists with adjectives
    emit "EN->PL A big dog and a small cat ate bread"
}

# ----------------------------- Main -----------------------------
main() {
    echo "# lexFlex Benchmark - expanded (~500 sentences)"
    echo "# Generated by scripts/generate_benchmark.sh"
    echo "# Exhausts: adjectives, nouns, verb inflections (gender/aspect),"
    echo "# enumerations (i + lists), negation, questions, temporals, quantifiers,"
    echo "# different frames, singular/plural, articles (EN), basic coordination."
    echo "# All material based on current data/lexicons and morphology RON."
    echo

    # PL->EN section
    generate_pl_basic
    generate_pl_enhanced

    # EN->PL section
    generate_en_basic
    generate_en_enhanced

    # Add a few more targeted stress tests for declension-like and aspect
    echo "PL->EN Tomek nie dał jabłka Izie wczoraj"
    echo "PL->EN Czy duży kot pił zimną wodę?"
    echo "PL->EN Iza i Mama zrobiły czerwony dom"
    echo "EN->PL The small blue book was on the table yesterday"
    echo "EN->PL Did many good students read the big book?"
}

if [[ -n "$OUTFILE" ]]; then
    main > "$OUTFILE"
    echo "Wrote $OUTFILE" >&2
else
    main
fi

echo "Tip: benchmark data lives under results/benchmarks/ or benchmarks/ (committed test sets)" >&2
