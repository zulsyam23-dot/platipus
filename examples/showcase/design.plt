// Palette and root surface shared by every showcase module.
style showcaseRoot {
    background: "#0b1220"
    color: "#e5edf7"
    padding: 24
}

theme dark {
    background: "#0b1220"
    surface: "#111a2e"
    surfaceAlt: "#17243b"
    border: "#263650"
    foreground: "#e5edf7"
    muted: "#94a3b8"
    accent: "#38bdf8"
    accentText: "#082f49"
    danger: "#fb7185"
    success: "#4ade80"
    radius: "14px"
    gap: "16px"
}

// Metadata example only: the showcase does not include an API server.
api ShowcaseApi {
    get snapshot from "/api/showcase"
    post reset from "/api/showcase/reset"
}
