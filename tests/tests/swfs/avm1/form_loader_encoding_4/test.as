// SWF Version 6

/*
 * This test is the fourth test out of a series of tests, testing the encoding the form loader uses to decode
 * text files.
 * This test tests whether text files with invalid UTF-8 characters loaded in an SWF file with SWF version 6
 * are decoded as Flash decodes them.
 * While Flash decodes the valid UTF-8 characters as UTF-8, the invalid characters are decoded as seemingly
 * random symbols. As of June 2024, Ruffle doesn't implement this. Therefore, this test is marked as known
 * failure.
 */

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
