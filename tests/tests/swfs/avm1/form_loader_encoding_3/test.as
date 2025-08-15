// SWF Version 6

/*
 * This test is the third test out of a series of tests, testing the encoding the form loader uses to decode
 * text files.
 * This test tests whether text files loaded with System#useCodepage set to true are decoded correctly.
 * While Flash uses the system codepage, Ruffle doesn't always have access to it, and it's not reliably the
 * correct encoding. Therefore, when System#useCodepage is set to true, text files in different encodings
 * should all be decoded correctly.
 */

System.useCodepage = true;

loadVariablesNum("UTF8.txt", 0);
loadVariablesNum("Iso.txt", 0);
loadVariablesNum("Shift Jis.txt", 0);

this.onEnterFrame = function() {
	if (this.utf8_var != undefined && this.iso_var != undefined && this.shift_var != undefined) {
		trace(this.utf8_var);
		trace(this.iso_var);
		trace(this.shift_var);
		this.onEnterFrame = null;
	}
};
