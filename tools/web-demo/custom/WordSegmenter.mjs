// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

import { lib } from "./index.mjs";

export default {
	"WordSegmenter.segment": {
		func: (model, text) => {
			var segmenter;
			switch (model) {
				case "Auto":
					segmenter = lib.WordSegmenter.createAuto();
					break;
				case "LSTM":
					segmenter = lib.WordSegmenter.createLstm();
					break;
				case "Dictionary":
					segmenter = lib.WordSegmenter.createDictionary();
			}
			
			let last = 0;
			const iter = segmenter.segment(text);

			const segments = [];
			
			while (true) {
				const next = iter.next();

				if (next === -1) {
					segments.push(text.slice(last));
					break;
				}

				segments.push(text.slice(last, next));
				last = next;
			}
			
			return segments.join(" . ");
		},
		expr: (model, text) => {
			switch (model) {
				case '"LSTM"':
					return 'WordSegmenter.createLstm().segment(text)';
				case '"Dictionary"':
					return 'WordSegmenter.createDictionary().segment(text)';
			}
			return 'WordSegmenter.createAuto().segment(text)';
		},
		funcName: "WordSegmenter.segment",
		parameters: [
			{
				name: "Model Type (Auto, LSTM, or Dictionary)",
				type: "string",
				typeUse: "string",
				defaultValue: "Auto"
			},
			{
				name: "text",
				type: "string",
				typeUse: "string",
				defaultValue: "โดยที่การยอมรับนับถือเกียรติศักดิ์ประจำตัว และสิทธิเท่าเทียมกันและโอนมิได้ของบรรดา สมาชิก ทั้ง หลายแห่งครอบครัว มนุษย์เป็นหลักมูลเหตุแห่งอิสรภาพ ความยุติธรรม และสันติภาพในโลก"
			}
		]
	}
};
