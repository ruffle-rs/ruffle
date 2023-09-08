package {
	public class Test {

	}
}

import com.ruffle.RuffleTest;
import flash.utils.getDefinitionByName;
import flash.utils.ByteArray;
import flash.system.System;
import avmplus.MyHelper;

var allFlags = [
		0,
		avmplus.INCLUDE_TRAITS | avmplus.HIDE_NSURI_METHODS,
		avmplus.INCLUDE_TRAITS | avmplus.INCLUDE_BASES,
		avmplus.INCLUDE_TRAITS | avmplus.INCLUDE_INTERFACES,
		avmplus.INCLUDE_TRAITS | avmplus.INCLUDE_VARIABLES,
		avmplus.INCLUDE_TRAITS | avmplus.INCLUDE_ACCESSORS,
		avmplus.INCLUDE_TRAITS | avmplus.INCLUDE_METHODS,
		avmplus.INCLUDE_TRAITS | avmplus.INCLUDE_METADATA,
		avmplus.INCLUDE_TRAITS | avmplus.INCLUDE_CONSTRUCTOR,
		avmplus.INCLUDE_TRAITS,
		avmplus.INCLUDE_TRAITS | avmplus.USE_ITRAITS,
		avmplus.INCLUDE_TRAITS | avmplus.HIDE_OBJECT
	];

for each (var flags in allFlags) {
	trace("Describing with flags: " + flags);
	printObject(MyHelper.descType(new RuffleTest("first", false), flags));
	trace();
}

function printObject(obj:Object, numTabs:int = 0):void {
	var tabs:String = "";
	for (var i:int = 0; i < numTabs; ++i) {
		tabs += "\t";
	}
	var keys = [];
	for (var k:* in obj) {
		keys.push(k);
	}
	keys.sort();
	for each (var key in keys) {
		var v:* = obj[key];
		row(tabs + key + " = " + v);
		if (v) {
			if (key == "methods" || key == "accessors" || key == "variables") {
				v.sort(function(m1, m2) {
						if (m1.name < m2.name) {
							return -1;

						}
						else if (m1.name > m2.name) {
							return 1;
						}
						else {
							return 0;
						}
					});

			}
			printObject(v, numTabs + 1);
		}
	}
}

function row(...cols):void {
	trace(cols.join(","));
}
