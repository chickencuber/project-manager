#!/bin/bash
echo -e "<!DOCTYPE html>\n<html lang=\"en\">\n  <head>\n    <link rel=\"stylesheet\" type=\"text/css\" href=\"style.css\">\n    <meta charset=\"utf-8\" />\n\n  </head>\n  <body>\n    <main>\n    </main>\n    <script src=\"main.js\"></script>\n  </body>\n</html>\n" > "./index.html"
touch "./style.css"
touch "./main.js"
git init
