use platipus_ir::IrModule;

pub fn render(module: &IrModule) -> String {
    format!(
        "<!doctype html>\n\
<html lang=\"en\">\n\
<head>\n\
<meta charset=\"utf-8\">\n\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
<title>{}</title>\n\
<link rel=\"stylesheet\" href=\"app.css\">\n\
</head>\n\
<body>\n\
<div id=\"root\"></div>\n\
<script type=\"module\" src=\"app.js\"></script>\n\
</body>\n\
</html>\n",
        module.name
    )
}
