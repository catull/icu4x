// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

import { RenderInfo, lib } from "../../gen/index.mjs";
import { TerminusRender } from "../../gen/rendering/rendering.mjs";
import beautify from 'js-beautify';

// Renders all termini into the class="container" element
Object.values(RenderInfo.termini).toSorted((a, b) => a.funcName < b.funcName ? -1 : 1).forEach((t) => {
	let details = document.createElement("details");
	let summary = document.createElement("summary");
	summary.innerHTML = `<code>${t.funcName}</code>`;
	details.appendChild(summary);
	details.appendChild(document.createElement("br"));
	details.appendChild(new TerminusRender(lib, () => { }, RenderInfo.termini[t.funcName],
		(code) => {
			code.innerText = beautify.js(code.innerText, {
				"indent_size": "2",
				"indent_char": " ",
				"break_chained_methods": true,
				// "brace_style": "collapse",
				"wrap_line_length": "45"
			});
		}));
	document.getElementsByClassName("container")[0].appendChild(details);
});
