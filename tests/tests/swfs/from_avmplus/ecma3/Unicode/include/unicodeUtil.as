/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
import com.adobe.test.Assert;
/*
Description:
Concatenate the unicode characters in the given range and test against it.
For each char in the given range:
  - search for the char in the test string
  - search for a string of 3 chars in the test string
  - match the char in the test string
  - split the test string around the char
  - insert a B, S, CS, WS char into the test string and split around the inserted char
  - replace the char with an empty string
  - replace the char with the first and last chars in the given unicode range

Modifications:
10/04/06 - cpeyer - change to single testcase for each range, to decrease # of
testcases.

*/

function testUnicodeRange(hexFrom, hexTo)
{
  // split the range into smaller more mangeable set
  var range = 250;
  for (var i = hexFrom; i <= hexTo; i= i+range+1 ) {
    var subHexFrom = i;
    var subHexTo = (i+range <= hexTo) ? i+range:hexTo;
    testUnicodeRangeHelper(subHexFrom, subHexTo);
  }
}

function testUnicodeRangeHelper(hexFrom, hexTo) {
  var offset:int = hexFrom;
  var testStr = "";
  for (var i = hexFrom; i <= hexTo; i++ ) {
    testStr += String.fromCharCode(i);
  }

  // The following vars hold the results of the unicode tests in the for loop
  var stringSearchResult:String = '';
  var string3SearchResult:String = '';
  var stringMatchResult:String = '';
  var stringSplitResult:String = '';


  for (var i = hexFrom; i <= hexTo; i++ ) {
    var charStr = String.fromCharCode(i);
    var charStrPattern = stringPatternFromCharCode(i);

    // 1. String.search()
    var searchExpect = i - offset;
    var searchResult:int = testStr.search(charStrPattern);
    if (searchExpect != searchResult) stringSearchResult += decimalToHexString(searchExpect) + ' ';

    // search a string of 3 chars
    var searchPattern = "";
    for (var j = i; j <= hexTo && j < i + 3; j++) {
      searchPattern += stringPatternFromCharCode(j);
    }
    searchResult = testStr.search(searchPattern);
    if (searchExpect != searchResult) string3SearchResult += decimalToHexString(searchExpect) + ' ';

    // 2. String.match()
    var matchResult:Array = testStr.match(charStrPattern);
    if (matchResult == null) {
      stringMatchResult += "Failed to find match: " + charStrPattern + " ";
    } else if (charStr != matchResult[0]) {
      stringMatchResult += "Failed to match chars: "+charStr + " with " + matchResult[0] + ' ';
    }

    // 3. String.split()
    var re = new RegExp("(" + charStrPattern + ")");
    var splitResult:Array = testStr.split(re);
    if (splitResult == null) {
      stringSplitResult += "Failed to split string on: " + charStr + ' ';
    } else {
      // test string before searched char
      if (testStr.substring(0,searchResult) != splitResult[0])
        stringSplitResult += "split failed before: " + charStr + ' ';
      // test searched char
      if (charStr != splitResult[1])
        stringSplitResult += "split failed on: " + charStr + ' ';
      // test string after searched char
      if (testStr.substring(searchResult + 1, testStr.length) != splitResult[2])
        stringSplitResult += "split failed after: " + charStr + ' ';
    }

  } // for loop

  // Output test results
  this.array[this.item++] = Assert.expectEq(
    "Unicode String.search from " + decimalToHexString(hexFrom) + " to " + decimalToHexString(hexFrom),
    '', stringSearchResult);
  this.array[this.item++] = Assert.expectEq(
    "Unicode String.search for 3 chars from " + decimalToHexString(hexFrom) + " to " + decimalToHexString(hexFrom),
    '', string3SearchResult);
  this.array[this.item++] = Assert.expectEq(
    "Unicode String.match", '', stringMatchResult);
  this.array[this.item++] = Assert.expectEq(
    "Unicode String.split", '', stringSplitResult);

  testReplace(hexFrom, hexTo);

  testSplitOnMark(testStr, B, "B");
  testSplitOnMark(testStr, S, "S");
  testSplitOnMark(testStr, CS, "CS");
  testSplitOnMark(testStr, WS, "WS");
}

function testReplace(hexFrom, hexTo)
{
  var offset:int = hexFrom;
  var testStr = "";
  for (var i = hexFrom; i <= hexTo; i++ ) {
    testStr += String.fromCharCode(i);
  }

  var stringReplaceResult:String = '';

  for (var i = hexFrom; i <= hexTo; i++ ) {
    var charStr = String.fromCharCode(i);
    var charStrPattern = stringPatternFromCharCode(i);

    // 4. String.replace()
    var index = i - offset;

    var replaceResult:String = testStr.replace(charStr, "");
    var replaceExpect = testStr.substring(0, index);
    replaceExpect += testStr.substring(index + 1, testStr.length);
    if (replaceExpect != replaceResult)
      stringReplaceResult += "Replace failed on: " + charStr + " ";

    // replace with first and last char in given Unicode range
    var firstLastChars = stringPatternFromCharCode(hexFrom) + stringPatternFromCharCode(hexTo);
    var replaceResult2:String = testStr.replace(charStr, firstLastChars);
    var replaceExpect2 = testStr.substring(0, index);
    replaceExpect2 += firstLastChars;
    replaceExpect2 += testStr.substring(index + 1, testStr.length);
    if (replaceExpect2 != replaceResult2)
      stringReplaceResult += "Replace failed swapping: " + firstLastChars + " ";
  }

  this.array[this.item++] = Assert.expectEq(
    "Unicode String.replace", '', stringReplaceResult);

}

function testSplitOnMark(testStr:String, markArray:Array, markArrayName:String) {
  var testSplitResult:String = ''; //holds results of splitting

  for (var i = 0; i < markArray.length; i++) {
    var mark = markArray[i];
    var markStr = String.fromCharCode(mark);
    var markStrPattern = stringPatternFromCharCode(mark);

    // insert the mark character into the middle of testStr
    var insertIndex = Math.floor(testStr.length / 2);
    var markedStr = testStr.substring(0, insertIndex);
    markedStr += markStr;
    markedStr += testStr.substring(insertIndex, testStr.length);

    // split around the mark

    var markRE = new RegExp("(" + markStrPattern + ")");
    var splitMarkedResult:Array = markedStr.split(markRE);
    var splitMessage = "Split on " + markArrayName + " mark " + decimalToHexString(mark);
    if (splitMarkedResult == null) {
      testSplitResult += 'array is null, expected not null!';
    } else {
      var markIndex = markedStr.indexOf(markStr, 0);

      // test segment before mark
      if (markedStr.substring(0, markIndex) != splitMarkedResult[0])
        testSplitResult += "Split failed before: " + decimalToHexString(mark);

      // test the mark we split on
      if (markedStr.substring(markIndex, markIndex + 1) != splitMarkedResult[1])
        testSplitResult += "Split failed on: " + decimalToHexString(mark);

      // test segment after mark
      var segmentEnd = markedStr.indexOf(markStr, markIndex + 1);
      if (segmentEnd == -1) {
        segmentEnd = markedStr.length;
      }
      if (markedStr.substring(markIndex + 1, segmentEnd) != splitMarkedResult[2])
        testSplitResult += "Split failed after: " + decimalToHexString(mark);
    } // else
  } // for

  this.array[this.item++] = Assert.expectEq(
    "Unicode Split on Mark", '', testSplitResult);

}

function regexpReserved(charCode) {
  return (charCode == 36)   // $
    || (charCode == 40)   // (
    || (charCode == 41)   // )
    || (charCode == 42)   // *
    || (charCode == 43)   // +
    || (charCode == 46)   // .
    || (charCode == 63)   // ?
    || (charCode == 91)   // [
    || (charCode == 92)   // \
    || (charCode == 94)   // ^
    || (charCode == 124); // |
}

var B = new Array(0x000A, // line feed
  0x000D, // carriage return
  0x001C,
  0x001D,
  0x001E,
  0x0085,
  0x2029);

var S = new Array(0x0009,
  0x000B,
  0x001F);

var CS = new Array(0x002C,
  0x002E,
  0x002F,
  0x003A,
  0x00A0,
  0x060C,
  0x2044,
  0xFE50,
  0xFE52,
  0xFE55,
  0xFF0C,
  0xFF0E,
  0xFF1A);

var WS = new Array(0x000C,
  0x0020,
  0x1680,
  0x180E,
  0x2000,
  0x2001,
  0x2002,
  0x2003,
  0x2004,
  0x2005,
  0x2006,
  0x2007,
  0x2008,
  0x2009,
  0x200A,
  0x2028,
  0x202F,
  0x205F,
  0x3000);

function stringPatternFromCharCode(charCode) {
  var result = "";
  if (regexpReserved(charCode)) {
    result += "\\";
  }
  result += String.fromCharCode(charCode);
  return result;
}

function decimalToHexString( n ) {
  n = Number( n );
  var h = "0x";

  for ( var i = 3; i >= 0; i-- ) {
    if ( n >= Math.pow(16, i) ){
      var t = Math.floor( n  / Math.pow(16, i));
      n -= t * Math.pow(16, i);
      if ( t >= 10 ) {
        if ( t == 10 ) { h += "A"; }
        if ( t == 11 ) { h += "B"; }
        if ( t == 12 ) { h += "C"; }
        if ( t == 13 ) { h += "D"; }
        if ( t == 14 ) { h += "E"; }
        if ( t == 15 ) { h += "F"; }
      } else {
        h += String( t );
      }
    } else {
      h += "0";
    }
  }
  return h;
}
