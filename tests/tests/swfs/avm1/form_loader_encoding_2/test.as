// SWF Version 5

/*
 * This test is the second test out of a series of tests, testing the encoding the form loader uses to decode
 * text files.
 * This test tests whether text files loaded in an SWF file with SWF version 5 are decoded as Windows-1252.
 * Flash decodes these files as Windows-1252 on Windows. On macOS, a different custom encoding is used (with
 * the letters with diacritics having the same encoding as in Windows-1252).
 */

// ActionScript code in Frame 1

loadVariablesNum("UTF8.txt", 0);
loadVariablesNum("Iso.txt", 0);
loadVariablesNum("Shift Jis.txt", 0);


// ActionScript code in Frame 3

trace(this.utf8_var);
trace(this.iso_var);
trace(this.shift_var);
stop();
