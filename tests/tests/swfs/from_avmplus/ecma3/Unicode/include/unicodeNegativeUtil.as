/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
import com.adobe.test.Assert;
function negativeTestUnicodeRange(hexFrom, hexTo, array, item) {
  // split the range into smaller more manageable set
  var range = 250;
  for (var i = hexFrom; i <= hexTo; i= i+range+1 ) {
    var subHexFrom = i;
    var subHexTo = (i+range <= hexTo)? i+range:hexTo;
    negativeTestUnicodeRangeHelper(subHexFrom, subHexTo);
  }
}

function negativeTestUnicodeRangeHelper(hexFrom, hexTo, array, item) {
  var offset:int = hexFrom;
  var testStr = "";
  var splitResult:Array = new Array();
  for (var i = hexFrom; i <= hexTo; i++ ) {
    testStr += String.fromCharCode(i);
    splitResult.push(String.fromCharCode(i));
  }

  negativeTestSearch(hexFrom, hexTo, array, item, testStr);
  negativeTestMatch(hexFrom, hexTo, array, item, testStr);
  negativeTestSplit(hexFrom, hexTo, array, item, testStr, splitResult);
  negativeTestReplace(hexFrom, hexTo, array, item, testStr);
}

function negativeTestSearch(hexFrom, hexTo, array, item, testStr)
{
  // String.search()
  var stringSearchResult:String = '';
  for (var i = hexFrom; i <= hexTo; i++ ) {
    var searchStr = String.fromCharCode(i) + String.fromCharCode(i);
    if (notReserved(i)) {
      var searchResult:int = testStr.search(searchStr);
      var searchExpect = -1
      if (searchExpect != searchResult)
        stringSearchResult += 'Expected: ' + searchExpect +' Found: ' + searchResult +' ';

      var pattern:RegExp = new RegExp(searchStr);
      var searchResult2:int = testStr.search(pattern);
      var searchExpect2 = -1;
      if (searchExpect2 != searchResult2)
        stringSearchResult += 'Expected: ' + searchExpect2 +' Found: ' + searchResult2 +' ';
    }
  }

  this.array[this.item++] = Assert.expectEq(
    "Negative String.search", '', stringSearchResult);

  var hexFromStr = decimalToHexString(hexFrom);
  var hexToStr = decimalToHexString(hexTo);

  //test matching undefined
  var searchResult:int = testStr.search(undefined);
  var searchExpect = -1;
  this.array[this.item++] = Assert.expectEq(
    hexFromStr + " to " + hexToStr +
    " String.search(undefined)", searchExpect, searchResult);

  //test matching with no parameter
  var searchResult2:int = testStr.search();
  var searchExpect2 = -1;
  this.array[this.item++] = Assert.expectEq(
    hexFromStr + " to " + hexToStr +
    " String.search()", searchExpect2, searchResult2);
}

function negativeTestMatch(hexFrom, hexTo, array, item, testStr)
{
  // String.match()
  //test matching a string that doesn't exist in the testStr
  var stringSearchResult:String = '';
  for (var i = hexFrom; i <= hexTo; i++ ) {
    if (notReserved(i)) {
      var matchResult:Array = testStr.match(String.fromCharCode(i)+String.fromCharCode(i));
      if (matchResult != null)
        stringSearchResult += 'Expected: null Found: ' + matchResult +' ';

      var pattern:RegExp = new RegExp(String.fromCharCode(i) + String.fromCharCode(i));
      var matchResult2:Array = testStr.match(pattern);
      if (matchResult2 != null)
        stringSearchResult += 'Expected: null Found: ' + matchResult2 +' ';
    }
  }

  this.array[this.item++] = Assert.expectEq(
    "Negative String.match", '', stringSearchResult);

  var hexFromStr = decimalToHexString(hexFrom);
  var hexToStr = decimalToHexString(hexTo);

  //test matching undefined
  var matchResult2:Array = testStr.match(undefined);
  this.array[this.item++] = Assert.expectEq(
    hexFromStr + " to " + hexToStr +
    " String.match(undefined)", null, matchResult2);

  //test matching with no parameter
  var matchResult3:Array = testStr.match();
  this.array[this.item++] = Assert.expectEq(
    hexFromStr + " to " + hexToStr +
    " String.match()", null, matchResult3);
}

function negativeTestSplit(hexFrom, hexTo, array, item, testStr, splitExpected)
{
  // String.split()
  //split on empty string
  var stringSplitResult:String = '';
  var splitDelimiter = "";
  var splitResult:Array = testStr.split(splitDelimiter);
  if (splitResult != null && splitExpected != null) {
    if (splitExpected.length != splitResult.length) {
      stringSplitResult += 'unexpected length - expected: '+splitExpected.length + 'Found: '+splitResult.length+' ';
    } else {
      for (var i = 0; i< splitExpected.length; i++) {
        if (splitExpected[i] != splitResult[i])
          stringSplitResult += 'mismatch - expected: '+splitExpected[i]+' Found: '+splitResult[i] + ' ';
      }
    }
  }
  else {
    stringSplitResult += 'result array null! ';
  }
  this.array[this.item++] = Assert.expectEq(
    "String.split('')", '', stringSplitResult);


  //split on empty regular expression
  stringSplitResult = '';
  var pattern:RegExp = new RegExp();
  var splitResult2:Array = testStr.split(pattern);
  if (splitResult2 != null && splitExpected != null) {
    if (splitExpected.length != splitResult2.length) {
      stringSplitResult += 'unexpected length - expected: '+splitExpected.length + 'Found: '+splitResult2.length+' ';
    } else {
      for (var i = 0; i< splitExpected.length; i++) {
        if (splitExpected[i] != splitResult2[i])
          stringSplitResult += 'mismatch - expected: '+splitExpected[i]+' Found: '+splitResult2[i] + ' ';
      }
    }
  }
  else {
    stringSplitResult += 'result array null! ';
  }
  this.array[this.item++] = Assert.expectEq(
    "String.split(new RegExp())", '', stringSplitResult);

  //split on empty regular expression
  stringSplitResult = '';
  var pattern:RegExp = new RegExp("");
  var splitResult3:Array = testStr.split(pattern);
  if (splitResult3 != null && splitExpected != null) {
    if (splitExpected.length != splitResult3.length) {
      stringSplitResult += 'unexpected length - expected: '+splitExpected.length + 'Found: '+splitResult3.length+' ';
    } else {
      for (var i = 0; i< splitExpected.length; i++) {
        if (splitExpected[i] != splitResult3[i])
          stringSplitResult += 'mismatch - expected: '+splitExpected[i]+' Found: '+splitResult3[i] + ' ';
      }
    }
  }
  else {
    stringSplitResult += 'result array null! ';
  }
  this.array[this.item++] = Assert.expectEq(
    "String.split(new RegExp(''))", '', stringSplitResult);


  //split on undefined
  var splitResult4:Array = testStr.split(undefined);
  if (splitResult4 != null) {
    this.array[this.item++] = Assert.expectEq(
      "String.split(undefined) result length", 1, splitResult4.length);

    if (splitResult4.length == 1) {
      this.array[this.item++] = Assert.expectEq(
        "String.split(undefined)[0]", testStr, splitResult4[0]);
    }
  }
}

function negativeTestReplace(hexFrom, hexTo, array, item, testStr)
{
  var replaceResult = testStr.replace(String.fromCharCode(hexFrom)+String.fromCharCode(hexTo));
  this.array[this.item++] = Assert.expectEq(
    "String.replace(" + decimalToHexString(hexFrom) + decimalToHexString(hexTo) + ")", testStr, replaceResult);
}

// return true if unicode is not a regexp reserved character
function notReserved(charCode) {
  return (charCode != 36)   //$
    && (charCode != 40)   //(
    && (charCode != 41)   //)
    && (charCode != 42)   //*
    && (charCode != 43)   //+
    && (charCode != 46)   //.
    && (charCode != 63)   //?
    && (charCode != 91)   //[
    && (charCode != 92)   //\
    && (charCode != 94)   //^
    && (charCode != 124); //|
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
