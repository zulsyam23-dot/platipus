// Shared presentation primitives used by the showcase sections.
component SectionIntro {
    input eyebrow: String = ""
    input title: String = ""
    input description: String = ""

    Column {
        gap: 5
        Text eyebrow {
            size: 11
            weight: 700
            color: "#38bdf8"
        }
        Heading title {
            level: 2
        }
        Text description {
            color: "#94a3b8"
        }
    }
}

component StatTile {
    input value: String = "0"
    input caption: String = ""

    Card {
        padding: 14
        Column {
            gap: 4
            Text caption {
                size: 12
                weight: 600
                color: "#94a3b8"
            }
            Text value {
                size: 24
                weight: 700
            }
        }
    }
}

component KeyValue {
    input key: String = ""
    input val: String = ""

    Row {
        gap: 10
        Text key {
            color: "#94a3b8"
        }
        Spacer { }
        Text val {
            weight: 600
        }
    }
}
