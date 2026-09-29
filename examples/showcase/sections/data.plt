// Collections, tables, data grids, and a state-driven tree.
component DataSection {
    state rows = [
        { name: "ann", team: "core", score: 91 },
        { name: "bob", team: "web", score: 78 },
        { name: "cyd", team: "core", score: 84 },
        { name: "dee", team: "docs", score: 66 }
    ]
    state order = "desc"
    state pageIndex = 0
    state shown = []
    state picked = "none"
    state treeExpanded = false

    fn refresh() {
        shown = page(sortBy(rows, "score", order), pageIndex, 2)
    }

    on mount { refresh() }

    Column {
        gap: 16
        SectionIntro {
            eyebrow: "DATA"
            title: "Lists, grids & trees"
            description: "Sort and page the sample records, select a row, or expand the composed tree."
        }

        Row {
            Button "sort desc" {
                on click {
                    order = "desc"
                    refresh()
                }
            }
            Button "sort asc" {
                on click {
                    order = "asc"
                    refresh()
                }
            }
            Button "next page" {
                on click {
                    pageIndex = pageIndex + 1
                    refresh()
                }
            }
            Button "first page" {
                on click {
                    pageIndex = 0
                    refresh()
                }
            }
        }

        Card {
            padding: 14
            Table {
                Header "name"
                Header "team"
                Header "score"
                for row in shown {
                    TableRow {
                        Cell row.name
                        Cell row.team
                        Cell row.score
                    }
                }
            }
        }

        SectionIntro {
            eyebrow: "ROW SELECTION"
            title: "DataGrid"
            description: "Select any record to see its name below."
        }
        DataGrid {
            for row in rows {
                TableRow {
                    Cell row.name
                    Cell row.team
                    on click { picked = row.name }
                }
            }
        }
        KeyValue { key: "Selected row" val: picked }

        Row {
            gap: 18
            Column {
                Text "List"
                List {
                    for row in rows { Text row.name }
                }
            }
            Column {
                Text "Tree · state + if"
                Button "toggle tree" { on click { treeExpanded = !treeExpanded } }
                Tree {
                    Text "teams"
                    if treeExpanded {
                        for row in rows { Text row.team }
                    }
                }
            }
        }
    }
}
