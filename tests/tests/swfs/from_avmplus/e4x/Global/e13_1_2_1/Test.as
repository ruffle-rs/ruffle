/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
-*- Mode: java; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 4 -*-
*
* ***** BEGIN LICENSE BLOCK *****
* Version: MPL 1.1/GPL 2.0/LGPL 2.1
*
* The contents of this file are subject to the Mozilla Public License Version
* 1.1 (the "License"); you may not use this file except in compliance with
* the License. You may obtain a copy of the License at
* http://www.mozilla.org/MPL/
*
* Software distributed under the License is distributed on an "AS IS" basis,
* WITHOUT WARRANTY OF ANY KIND, either express or implied. See the License
* for the specific language governing rights and limitations under the
* License.
*
* The Original Code is Rhino code, released
* May 6, 1999.
*
* The Initial Developer of the Original Code is
* Netscape Communications Corporation.
* Portions created by the Initial Developer are Copyright (C) 1997-2000
* the Initial Developer. All Rights Reserved.
*
* Contributor(s):
*   Igor Bukanov
*
* Alternatively, the contents of this file may be used under the terms of
* either the GNU General Public License Version 2 or later (the "GPL"), or
* the GNU Lesser General Public License Version 2.1 or later (the "LGPL"),
* in which case the provisions of the GPL or the LGPL are applicable instead
* of those above. If you wish to allow use of your version of this file only
* under the terms of either the GPL or the LGPL, and not to allow others to
* use your version of this file under the terms of the MPL, indicate your
* decision by deleting the provisions above and replace them with the notice
* and other provisions required by the GPL or the LGPL. If you do not delete
* the provisions above, a recipient may use your version of this file under
* the terms of any one of the MPL, the GPL or the LGPL.
*
 * ***** END LICENSE BLOCK ***** */
package {
public class Test {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

function START(summary)
{
      // print out bugnumber

     /*if ( BUGNUMBER ) {
              writeLineToLog ("BUGNUMBER: " + BUGNUMBER );
      }*/
    XML.setSettings (null);
    testcases = new Array();

    // text field for results
    tc = 0;
    /*this.addChild ( tf );
    tf.x = 30;
    tf.y = 50;
    tf.width = 200;
    tf.height = 400;*/

    //_print(summary);
    var summaryParts = summary.split(" ");
    //_print("section: " + summaryParts[0] + "!");
    //fileName = summaryParts[0];

}

function TEST(section, expected, actual)
{
    AddTestCase(section, expected, actual);
}
 

function TEST_XML(section, expected, actual)
{
  var actual_t = typeof actual;
  var expected_t = typeof expected;

  if (actual_t != "xml") {
    // force error on type mismatch
    TEST(section, new XML(), actual);
    return;
  }

  if (expected_t == "string") {

    TEST(section, expected, actual.toXMLString());
  } else if (expected_t == "number") {

    TEST(section, String(expected), actual.toXMLString());
  } else {
    reportFailure ("", 'Bad TEST_XML usage: type of expected is "+expected_t+", should be number or string');
  }
}

function reportFailure (section, msg)
{
  trace("~FAILURE: " + section + " | " + msg);
}

function AddTestCase( description, expect, actual ) {
   testcases[tc++] = Assert.expectEq(description, "|"+expect+"|", "|"+actual+"|" );
}

function myGetNamespace (obj, ns) {
    if (ns != undefined) {
        return obj.namespace(ns);
    } else {
        return obj.namespace();
    }
}




function NL()
{
  //return java.lang.System.getProperty("line.separator");
  return "\n";
}


function BUG(arg){
  // nothing here
}

function END()
{
    //test();
}

START("13.1.2.1 - isXMLName()");
 
TEST(1, true, typeof isXMLName == "function");
 

// Check converting to string
var object = { toString: function() { return "text"; } };

TEST(2, true, isXMLName(object));

// Check various cases of http://w3.org/TR/xml-names11/#NT-NCName

TEST(5, false, isXMLName(""));

TEST(5.1, false, isXMLName(null));

TEST(5.2, false, isXMLName());

TEST(5.3, false, isXMLName(undefined));

// Check various cases of http://w3.org/TR/xml-names11/#NT-NCName

TEST(5, false, isXMLName(""));

var BEGIN = 0x1;
var OTHER = 0x2;
var chars = init();
var marker;

// Letter

// Letter::= BaseChar | Ideographic

marker = BEGIN | OTHER;

// BaseChar

markRange(chars, 0x0041, 0x005A, marker);
markRange(chars, 0x0061, 0x007A, marker);
markRange(chars, 0x00C0, 0x00D6, marker);
markRange(chars, 0x00D8, 0x00F6, marker);
markRange(chars, 0x00F8, 0x00FF, marker);
markRange(chars, 0x0100, 0x0131, marker);
markRange(chars, 0x0134, 0x013E, marker);
markRange(chars, 0x0141, 0x0148, marker);
markRange(chars, 0x014A, 0x017E, marker);
markRange(chars, 0x0180, 0x01C3, marker);
markRange(chars, 0x01CD, 0x01F0, marker);
markRange(chars, 0x01F4, 0x01F5, marker);
markRange(chars, 0x01FA, 0x0217, marker);
markRange(chars, 0x0250, 0x02A8, marker);
markRange(chars, 0x02BB, 0x02C1, marker);
markRange(chars, 0x0386, 0x0386, marker);
markRange(chars, 0x0388, 0x038A, marker);
markRange(chars, 0x038C, 0x038C, marker);
markRange(chars, 0x038E, 0x03A1, marker);
markRange(chars, 0x03A3, 0x03CE, marker);
markRange(chars, 0x03D0, 0x03D6, marker);
markRange(chars, 0x03DA, 0x03DA, marker);
markRange(chars, 0x03DC, 0x03DC, marker);
markRange(chars, 0x03DE, 0x03DE, marker);
markRange(chars, 0x03E0, 0x03E0, marker);
markRange(chars, 0x03E2, 0x03F3, marker);
markRange(chars, 0x0401, 0x040C, marker);
markRange(chars, 0x040E, 0x044F, marker);
markRange(chars, 0x0451, 0x045C, marker);
markRange(chars, 0x045E, 0x0481, marker);
markRange(chars, 0x0490, 0x04C4, marker);
markRange(chars, 0x04C7, 0x04C8, marker);
markRange(chars, 0x04CB, 0x04CC, marker);
markRange(chars, 0x04D0, 0x04EB, marker);
markRange(chars, 0x04EE, 0x04F5, marker);
markRange(chars, 0x04F8, 0x04F9, marker);
markRange(chars, 0x0531, 0x0556, marker);
markRange(chars, 0x0559, 0x0559, marker);
markRange(chars, 0x0561, 0x0586, marker);
markRange(chars, 0x05D0, 0x05EA, marker);
markRange(chars, 0x05F0, 0x05F2, marker);
markRange(chars, 0x0621, 0x063A, marker);
markRange(chars, 0x0641, 0x064A, marker);
markRange(chars, 0x0671, 0x06B7, marker);
markRange(chars, 0x06BA, 0x06BE, marker);
markRange(chars, 0x06C0, 0x06CE, marker);
markRange(chars, 0x06D0, 0x06D3, marker);
markRange(chars, 0x06D5, 0x06D5, marker);
markRange(chars, 0x06E5, 0x06E6, marker);
markRange(chars, 0x0905, 0x0939, marker);
markRange(chars, 0x093D, 0x093D, marker);
markRange(chars, 0x0958, 0x0961, marker);
markRange(chars, 0x0985, 0x098C, marker);
markRange(chars, 0x098F, 0x0990, marker);
markRange(chars, 0x0993, 0x09A8, marker);
markRange(chars, 0x09AA, 0x09B0, marker);
markRange(chars, 0x09B2, 0x09B2, marker);
markRange(chars, 0x09B6, 0x09B9, marker);
markRange(chars, 0x09DC, 0x09DD, marker);
markRange(chars, 0x09DF, 0x09E1, marker);
markRange(chars, 0x09F0, 0x09F1, marker);
markRange(chars, 0x0A05, 0x0A0A, marker);
markRange(chars, 0x0A0F, 0x0A10, marker);
markRange(chars, 0x0A13, 0x0A28, marker);
markRange(chars, 0x0A2A, 0x0A30, marker);
markRange(chars, 0x0A32, 0x0A33, marker);
markRange(chars, 0x0A35, 0x0A36, marker);
markRange(chars, 0x0A38, 0x0A39, marker);
markRange(chars, 0x0A59, 0x0A5C, marker);
markRange(chars, 0x0A5E, 0x0A5E, marker);
markRange(chars, 0x0A72, 0x0A74, marker);
markRange(chars, 0x0A85, 0x0A8B, marker);
markRange(chars, 0x0A8D, 0x0A8D, marker);
markRange(chars, 0x0A8F, 0x0A91, marker);
markRange(chars, 0x0A93, 0x0AA8, marker);
markRange(chars, 0x0AAA, 0x0AB0, marker);
markRange(chars, 0x0AB2, 0x0AB3, marker);
markRange(chars, 0x0AB5, 0x0AB9, marker);
markRange(chars, 0x0ABD, 0x0ABD, marker);
markRange(chars, 0x0AE0, 0x0AE0, marker);
markRange(chars, 0x0B05, 0x0B0C, marker);
markRange(chars, 0x0B0F, 0x0B10, marker);
markRange(chars, 0x0B13, 0x0B28, marker);
markRange(chars, 0x0B2A, 0x0B30, marker);
markRange(chars, 0x0B32, 0x0B33, marker);
markRange(chars, 0x0B36, 0x0B39, marker);
markRange(chars, 0x0B3D, 0x0B3D, marker);
markRange(chars, 0x0B5C, 0x0B5D, marker);
markRange(chars, 0x0B5F, 0x0B61, marker);
markRange(chars, 0x0B85, 0x0B8A, marker);
markRange(chars, 0x0B8E, 0x0B90, marker);
markRange(chars, 0x0B92, 0x0B95, marker);
markRange(chars, 0x0B99, 0x0B9A, marker);
markRange(chars, 0x0B9C, 0x0B9C, marker);
markRange(chars, 0x0B9E, 0x0B9F, marker);
markRange(chars, 0x0BA3, 0x0BA4, marker);
markRange(chars, 0x0BA8, 0x0BAA, marker);
markRange(chars, 0x0BAE, 0x0BB5, marker);
markRange(chars, 0x0BB7, 0x0BB9, marker);
markRange(chars, 0x0C05, 0x0C0C, marker);
markRange(chars, 0x0C0E, 0x0C10, marker);
markRange(chars, 0x0C12, 0x0C28, marker);
markRange(chars, 0x0C2A, 0x0C33, marker);
markRange(chars, 0x0C35, 0x0C39, marker);
markRange(chars, 0x0C60, 0x0C61, marker);
markRange(chars, 0x0C85, 0x0C8C, marker);
markRange(chars, 0x0C8E, 0x0C90, marker);
markRange(chars, 0x0C92, 0x0CA8, marker);
markRange(chars, 0x0CAA, 0x0CB3, marker);
markRange(chars, 0x0CB5, 0x0CB9, marker);
markRange(chars, 0x0CDE, 0x0CDE, marker);
markRange(chars, 0x0CE0, 0x0CE1, marker);
markRange(chars, 0x0D05, 0x0D0C, marker);
markRange(chars, 0x0D0E, 0x0D10, marker);
markRange(chars, 0x0D12, 0x0D28, marker);
markRange(chars, 0x0D2A, 0x0D39, marker);
markRange(chars, 0x0D60, 0x0D61, marker);
markRange(chars, 0x0E01, 0x0E2E, marker);
markRange(chars, 0x0E30, 0x0E30, marker);
markRange(chars, 0x0E32, 0x0E33, marker);
markRange(chars, 0x0E40, 0x0E45, marker);
markRange(chars, 0x0E81, 0x0E82, marker);
markRange(chars, 0x0E84, 0x0E84, marker);
markRange(chars, 0x0E87, 0x0E88, marker);
markRange(chars, 0x0E8A, 0x0E8A, marker);
markRange(chars, 0x0E8D, 0x0E8D, marker);
markRange(chars, 0x0E94, 0x0E97, marker);
markRange(chars, 0x0E99, 0x0E9F, marker);
markRange(chars, 0x0EA1, 0x0EA3, marker);
markRange(chars, 0x0EA5, 0x0EA5, marker);
markRange(chars, 0x0EA7, 0x0EA7, marker);
markRange(chars, 0x0EAA, 0x0EAB, marker);
markRange(chars, 0x0EAD, 0x0EAE, marker);
markRange(chars, 0x0EB0, 0x0EB0, marker);
markRange(chars, 0x0EB2, 0x0EB3, marker);
markRange(chars, 0x0EBD, 0x0EBD, marker);
markRange(chars, 0x0EC0, 0x0EC4, marker);
markRange(chars, 0x0F40, 0x0F47, marker);
markRange(chars, 0x0F49, 0x0F69, marker);
markRange(chars, 0x10A0, 0x10C5, marker);
markRange(chars, 0x10D0, 0x10F6, marker);
markRange(chars, 0x1100, 0x1100, marker);
markRange(chars, 0x1102, 0x1103, marker);
markRange(chars, 0x1105, 0x1107, marker);
markRange(chars, 0x1109, 0x1109, marker);
markRange(chars, 0x110B, 0x110C, marker);
markRange(chars, 0x110E, 0x1112, marker);
markRange(chars, 0x113C, 0x113C, marker);
markRange(chars, 0x113E, 0x113E, marker);
markRange(chars, 0x1140, 0x1140, marker);
markRange(chars, 0x114C, 0x114C, marker);
markRange(chars, 0x114E, 0x114E, marker);
markRange(chars, 0x1150, 0x1150, marker);
markRange(chars, 0x1154, 0x1155, marker);
markRange(chars, 0x1159, 0x1159, marker);
markRange(chars, 0x115F, 0x1161, marker);
markRange(chars, 0x1163, 0x1163, marker);
markRange(chars, 0x1165, 0x1165, marker);
markRange(chars, 0x1167, 0x1167, marker);
markRange(chars, 0x1169, 0x1169, marker);
markRange(chars, 0x116D, 0x116E, marker);
markRange(chars, 0x1172, 0x1173, marker);
markRange(chars, 0x1175, 0x1175, marker);
markRange(chars, 0x119E, 0x119E, marker);
markRange(chars, 0x11A8, 0x11A8, marker);
markRange(chars, 0x11AB, 0x11AB, marker);
markRange(chars, 0x11AE, 0x11AF, marker);
markRange(chars, 0x11B7, 0x11B8, marker);
markRange(chars, 0x11BA, 0x11BA, marker);
markRange(chars, 0x11BC, 0x11C2, marker);
markRange(chars, 0x11EB, 0x11EB, marker);
markRange(chars, 0x11F0, 0x11F0, marker);
markRange(chars, 0x11F9, 0x11F9, marker);
markRange(chars, 0x1E00, 0x1E9B, marker);
markRange(chars, 0x1EA0, 0x1EF9, marker);
markRange(chars, 0x1F00, 0x1F15, marker);
markRange(chars, 0x1F18, 0x1F1D, marker);
markRange(chars, 0x1F20, 0x1F45, marker);
markRange(chars, 0x1F48, 0x1F4D, marker);
markRange(chars, 0x1F50, 0x1F57, marker);
markRange(chars, 0x1F59, 0x1F59, marker);
markRange(chars, 0x1F5B, 0x1F5B, marker);
markRange(chars, 0x1F5D, 0x1F5D, marker);
markRange(chars, 0x1F5F, 0x1F7D, marker);
markRange(chars, 0x1F80, 0x1FB4, marker);
markRange(chars, 0x1FB6, 0x1FBC, marker);
markRange(chars, 0x1FBE, 0x1FBE, marker);
markRange(chars, 0x1FC2, 0x1FC4, marker);
markRange(chars, 0x1FC6, 0x1FCC, marker);
markRange(chars, 0x1FD0, 0x1FD3, marker);
markRange(chars, 0x1FD6, 0x1FDB, marker);
markRange(chars, 0x1FE0, 0x1FEC, marker);
markRange(chars, 0x1FF2, 0x1FF4, marker);
markRange(chars, 0x1FF6, 0x1FFC, marker);
markRange(chars, 0x2126, 0x2126, marker);
markRange(chars, 0x212A, 0x212B, marker);
markRange(chars, 0x212E, 0x212E, marker);
markRange(chars, 0x2180, 0x2182, marker);
markRange(chars, 0x3041, 0x3094, marker);
markRange(chars, 0x30A1, 0x30FA, marker);
markRange(chars, 0x3105, 0x312C, marker);
markRange(chars, 0xAC00, 0xD7A3, marker);

// Ideographic

markRange(chars, 0x4E00, 0x9FA5, marker);
markRange(chars, 0x3007, 0x3007, marker);
markRange(chars, 0x3021, 0x3029, marker);

// Digit

marker = OTHER;

markRange(chars, 0x0030, 0x0039, marker);
markRange(chars, 0x0660, 0x0669, marker);
markRange(chars, 0x06F0, 0x06F9, marker);
markRange(chars, 0x0966, 0x096F, marker);
markRange(chars, 0x09E6, 0x09EF, marker);
markRange(chars, 0x0A66, 0x0A6F, marker);
markRange(chars, 0x0AE6, 0x0AEF, marker);
markRange(chars, 0x0B66, 0x0B6F, marker);
markRange(chars, 0x0BE7, 0x0BEF, marker);
markRange(chars, 0x0C66, 0x0C6F, marker);
markRange(chars, 0x0CE6, 0x0CEF, marker);
markRange(chars, 0x0D66, 0x0D6F, marker);
markRange(chars, 0x0E50, 0x0E59, marker);
markRange(chars, 0x0ED0, 0x0ED9, marker);
markRange(chars, 0x0F20, 0x0F29, marker);

// "Other NameChars"

markRange(chars, 0x2e, 0x2e, marker);
markRange(chars, 0x2d, 0x2d, marker);

marker = BEGIN | OTHER;

markRange(chars, 0x5f, 0x5f, marker);

// e4x excludes ':'

// CombiningChar

marker = OTHER;

markRange(chars, 0x0300, 0x0345, marker);
markRange(chars, 0x0360, 0x0361, marker);
markRange(chars, 0x0483, 0x0486, marker);
markRange(chars, 0x0591, 0x05A1, marker);
markRange(chars, 0x05A3, 0x05B9, marker);
markRange(chars, 0x05BB, 0x05BD, marker);
markRange(chars, 0x05BF, 0x05BF, marker);
markRange(chars, 0x05C1, 0x05C2, marker);
markRange(chars, 0x05C4, 0x05C4, marker);
markRange(chars, 0x064B, 0x0652, marker);
markRange(chars, 0x0670, 0x0670, marker);
markRange(chars, 0x06D6, 0x06DC, marker);
markRange(chars, 0x06DD, 0x06DF, marker);
markRange(chars, 0x06E0, 0x06E4, marker);
markRange(chars, 0x06E7, 0x06E8, marker);
markRange(chars, 0x06EA, 0x06ED, marker);
markRange(chars, 0x0901, 0x0903, marker);
markRange(chars, 0x093C, 0x093C, marker);
markRange(chars, 0x093E, 0x094C, marker);
markRange(chars, 0x094D, 0x094D, marker);
markRange(chars, 0x0951, 0x0954, marker);
markRange(chars, 0x0962, 0x0963, marker);
markRange(chars, 0x0981, 0x0983, marker);
markRange(chars, 0x09BC, 0x09BC, marker);
markRange(chars, 0x09BE, 0x09BE, marker);
markRange(chars, 0x09BF, 0x09BF, marker);
markRange(chars, 0x09C0, 0x09C4, marker);
markRange(chars, 0x09C7, 0x09C8, marker);
markRange(chars, 0x09CB, 0x09CD, marker);
markRange(chars, 0x09D7, 0x09D7, marker);
markRange(chars, 0x09E2, 0x09E3, marker);
markRange(chars, 0x0A02, 0x0A02, marker);
markRange(chars, 0x0A3C, 0x0A3C, marker);
markRange(chars, 0x0A3E, 0x0A3E, marker);
markRange(chars, 0x0A3F, 0x0A3F, marker);
markRange(chars, 0x0A40, 0x0A42, marker);
markRange(chars, 0x0A47, 0x0A48, marker);
markRange(chars, 0x0A4B, 0x0A4D, marker);
markRange(chars, 0x0A70, 0x0A71, marker);
markRange(chars, 0x0A81, 0x0A83, marker);
markRange(chars, 0x0ABC, 0x0ABC, marker);
markRange(chars, 0x0ABE, 0x0AC5, marker);
markRange(chars, 0x0AC7, 0x0AC9, marker);
markRange(chars, 0x0ACB, 0x0ACD, marker);
markRange(chars, 0x0B01, 0x0B03, marker);
markRange(chars, 0x0B3C, 0x0B3C, marker);
markRange(chars, 0x0B3E, 0x0B43, marker);
markRange(chars, 0x0B47, 0x0B48, marker);
markRange(chars, 0x0B4B, 0x0B4D, marker);
markRange(chars, 0x0B56, 0x0B57, marker);
markRange(chars, 0x0B82, 0x0B83, marker);
markRange(chars, 0x0BBE, 0x0BC2, marker);
markRange(chars, 0x0BC6, 0x0BC8, marker);
markRange(chars, 0x0BCA, 0x0BCD, marker);
markRange(chars, 0x0BD7, 0x0BD7, marker);
markRange(chars, 0x0C01, 0x0C03, marker);
markRange(chars, 0x0C3E, 0x0C44, marker);
markRange(chars, 0x0C46, 0x0C48, marker);
markRange(chars, 0x0C4A, 0x0C4D, marker);
markRange(chars, 0x0C55, 0x0C56, marker);
markRange(chars, 0x0C82, 0x0C83, marker);
markRange(chars, 0x0CBE, 0x0CC4, marker);
markRange(chars, 0x0CC6, 0x0CC8, marker);
markRange(chars, 0x0CCA, 0x0CCD, marker);
markRange(chars, 0x0CD5, 0x0CD6, marker);
markRange(chars, 0x0D02, 0x0D03, marker);
markRange(chars, 0x0D3E, 0x0D43, marker);
markRange(chars, 0x0D46, 0x0D48, marker);
markRange(chars, 0x0D4A, 0x0D4D, marker);
markRange(chars, 0x0D57, 0x0D57, marker);
markRange(chars, 0x0E31, 0x0E31, marker);
markRange(chars, 0x0E34, 0x0E3A, marker);
markRange(chars, 0x0E47, 0x0E4E, marker);
markRange(chars, 0x0EB1, 0x0EB1, marker);
markRange(chars, 0x0EB4, 0x0EB9, marker);
markRange(chars, 0x0EBB, 0x0EBC, marker);
markRange(chars, 0x0EC8, 0x0ECD, marker);
markRange(chars, 0x0F18, 0x0F19, marker);
markRange(chars, 0x0F35, 0x0F35, marker);
markRange(chars, 0x0F37, 0x0F37, marker);
markRange(chars, 0x0F39, 0x0F39, marker);
markRange(chars, 0x0F3E, 0x0F3E, marker);
markRange(chars, 0x0F3F, 0x0F3F, marker);
markRange(chars, 0x0F71, 0x0F84, marker);
markRange(chars, 0x0F86, 0x0F8B, marker);
markRange(chars, 0x0F90, 0x0F95, marker);
markRange(chars, 0x0F97, 0x0F97, marker);
markRange(chars, 0x0F99, 0x0FAD, marker);
markRange(chars, 0x0FB1, 0x0FB7, marker);
markRange(chars, 0x0FB9, 0x0FB9, marker);
markRange(chars, 0x20D0, 0x20DC, marker);
markRange(chars, 0x20E1, 0x20E1, marker);
markRange(chars, 0x302A, 0x302F, marker);
markRange(chars, 0x3099, 0x3099, marker);
markRange(chars, 0x309A, 0x309A, marker);

// Extender

markRange(chars, 0x00B7, 0x00B7, marker);
markRange(chars, 0x02D0, 0x02D0, marker);
markRange(chars, 0x02D1, 0x02D1, marker);
markRange(chars, 0x0387, 0x0387, marker);
markRange(chars, 0x0640, 0x0640, marker);
markRange(chars, 0x0E46, 0x0E46, marker);
markRange(chars, 0x0EC6, 0x0EC6, marker);
markRange(chars, 0x3005, 0x3005, marker);
markRange(chars, 0x3031, 0x3035, marker);
markRange(chars, 0x309D, 0x309E, marker);
markRange(chars, 0x30FC, 0x30FE, marker);

TEST(6, '', testIsXMLName(chars));

TEST(7, true, (testIsXMLName(chars) == ''));

END();


// Utilities

function markRange(buffer, start, end, marker)
{
    for (var i = start; i <= end; i++)
    {
        buffer[i] |= marker;
    }
}

function init()
{
    var length = 0xFFFF + 1;
    var chars = new Array(length);
    for (var i = 0; i < length; i++)
    {
        chars[i] = 0;
    }
    return chars;
}

function testIsXMLName(buffer)
{
    var nl       = NL();
    var result   = '';
    var length   = buffer.length;

    var rangestart = null;
    var rangeend   = null;
    var rangemessage = '';

    for (var i = 0; i < length; i++)
    {
        var message = '';
        var c       = String.fromCharCode(i);
        var marker  = buffer[i];

        var namestart = isXMLName(c + 'x');
        var nameother = isXMLName('x' + c);
  
        if (marker == 0 && namestart)
        {
            message += ': Invalid char accepted as start ';
        }

        if (marker == 0 && nameother)
        {
            message += ': Invalid Char accepted as other ';
        }

        if ((marker & BEGIN) && !namestart)
        {
            message += ': Start char not accepted';
        }

        if ((marker & OTHER) && !nameother)
        {
            message += ': Other char not accepted';
        }


        if (rangemessage && !message)
        {
            // no current error, previous error
            // output previous error range
            result += '[' + rangestart + '-' + rangeend + '] ' +
                rangemessage + nl;
            rangemessage = rangestart = rangeend = null;
        }
        else if (!rangemessage && message)
        {
            // current error, no previous error
            // start new error range
            rangemessage = message;
            rangestart = rangeend = formatChar(c);
        }
        else if (rangemessage && message)
        {
            if (rangemessage == message)
            {
                // current error same as previous
                // continue previous error range
                rangeend = formatChar(c);
            }
            else
            {
                // different error, output range
                result += '[' + rangestart + '-' + rangeend + '] ' +
                    rangemessage + nl;
                rangemessage = message;
                rangestart = rangeend = formatChar(c);
            }
        }
    }

    if (rangemessage)
    {
        result += '[' + rangestart + '-' + rangeend + '] ' +
            rangemessage + nl;
    }

    return result;
}

function formatChar(c)
{
    var s = '0x' + c.charCodeAt(0).toString(16).toUpperCase();
    return s;
}
