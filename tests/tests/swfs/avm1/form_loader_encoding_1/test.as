// SWF Version 6

/*
 * This test is the first test out of a series of tests, testing the encoding the form loader uses to decode
 * text files.
 * This test tests whether a UTF-8 text file loaded in an SWF file with SWF version 6 is decoded as UTF-8.
 * Flash decodes invalid UTF-8 characters (such as some characters encoded in ISO Latin-1) differently, and
 * as of June 2024, Ruffle doesn't implement this (see the test form_loader_encoding_4). Therefore, this test
 * only tests a UTF-8 file.
 */

loadVariablesNum("UTF8.txt", 0);

this.onEnterFrame = function() {
	if (this.utf8_var != undefined) {
		trace(this.utf8_var);
		this.onEnterFrame = null;
	}
};
