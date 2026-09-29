// Browser controls, two-way bindings, persistence, and form submission.
component FormSection {
    state notice: String = ""
    state name = "ada"
    state savedName = null
    state bio = ""
    state plan = "pro"
    state volume = 5
    state agreed = false
    state notify = true
    state when = "2026-09-29"
    state clock = "09:30"
    state accent = "#3399ff"
    state attachment = ""

    on mount {
        savedName = load("local", "showcase.name")
        if savedName != null {
            name = savedName
        }
    }

    Form {
        novalidate: true
        on submit {
            event.preventDefault()
            name = "ada-" + volume
            store("local", "showcase.name", name)
            notice = "submitted as " + name
        }

        Column {
            gap: 14
            SectionIntro {
                eyebrow: "INPUTS & FORMS"
                title: "Form controls"
                description: "Edit controls, try two-way bindings, then submit to save a name locally."
            }

            Grid columns: 2 {
                Field {
                    Text "Name"
                    Input {
                        placeholder: "your name"
                        bind value: name
                        on keydown {
                            notice = "key: " + event.key
                        }
                        on blur {
                            store("local", "showcase.name", name)
                            notice = "name saved locally"
                        }
                    }
                }
                Field {
                    Text "Bio"
                    Textarea {
                        placeholder: "say something"
                        bind value: bio
                    }
                }
                Field {
                    Text "Plan"
                    Select {
                        bind value: plan
                        Option "free" { value: "free" }
                        Option "pro" { value: "pro" }
                        Option "team" { value: "team" }
                    }
                }
                Field {
                    Text "Volume"
                    Slider {
                        min: 0
                        max: 10
                        step: 1
                        bind value: volume
                    }
                    Row {
                        Text volume
                        Button "volume up" {
                            on click {
                                if volume < 10 {
                                    volume = volume + 1
                                }
                            }
                        }
                    }
                }
                Field {
                    Text "Date / time"
                    Row {
                        Date { bind value: when }
                        Time { bind value: clock }
                    }
                }
                Field {
                    Text "Accent color"
                    Color { bind value: accent }
                }
                Field {
                    Text "Attachment"
                    Text "The browser reports the selected file path."
                    File {
                        accept: ".txt"
                        on change { attachment = event.value }
                    }
                    Text attachment
                }
            }

            Row {
                gap: 10
                Checkbox { bind checked: agreed }
                Text "I agree"
                Switch { bind checked: notify }
                Text "Notifications"
                Spacer { }
                Button "submit form" { }
            }
            Text notice
            Text name
        }
    }
}
