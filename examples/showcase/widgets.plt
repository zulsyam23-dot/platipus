// Reusable pieces the showcase app is built from. This file exists to exercise
// the module loader: it declares no `app`, and everything it declares is
// inlined into the importing program and then referred to by bare name.
component StatTile {
    input value: String = "0"
    input caption: String = ""

    Card {
        padding: 14
        Column {
            gap: 2
            Text caption
            Text value
        }
    }
}

component KeyValue {
    input key: String = ""
    input val: String = ""

    Row {
        gap: 10
        Text key
        Text val
    }
}

component Section {
    input title: String = ""
    input note: String = ""

    Column {
        gap: 8
        Heading title
        if note != "" {
            Text note
        }
    }
}
