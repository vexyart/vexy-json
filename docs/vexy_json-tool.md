---
layout: default
title: "Vexy JSON Parser"
description: "Interactive Vexy JSON parser with comments and flexible syntax"
nav_order: 10
permalink: /vexy_json-tool/
---

# Vexy JSON Interactive Parser

<iframe src="{{ '/tool.html' | relative_url }}" 
        width="100%" 
        height="900px" 
        style="border: none; border-radius: 8px; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);"
        loading="lazy">
</iframe>

[Open in new window]({{ '/tool.html' | relative_url }}){: .btn .btn-primary target="_blank"}

## Features

This interactive tool demonstrates all Vexy JSON forgiving features:
- Comments (`//`, `#`, `/* */`)
- Unquoted keys and single quotes
- Trailing commas
- Implicit top-level objects/arrays
- Newlines as comma separators