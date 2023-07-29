/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

/*
 *  String.match( regexp )
 *
 *  If regexp is not an object of type RegExp, it is replaced with result
 *  of the expression new RegExp(regexp). Let string denote the result of
 *  converting the this value to a string.  If regexp.global is false,
 *  return the result obtained by invoking RegExp.prototype.exec (see
 *  section 15.7.5.3) on regexp with string as parameter.
 *
 *  Otherwise, set the regexp.lastIndex property to 0 and invoke
 *  RegExp.prototype.exec repeatedly until there is no match. If there is a
 *  match with an empty string (in other words, if the value of
 *  regexp.lastIndex is left unchanged) increment regexp.lastIndex by 1.
 *  The value returned is an array with the properties 0 through n-1
 *  corresponding to the first element of the result of each matching
 *  invocation of RegExp.prototype.exec.
 *
 *  Note that the match function is intentionally generic; it does not
 *  require that its this value be a string object.  Therefore, it can be
 *  transferred to other kinds of objects for use as a method.
 */

//     var SECTION = "String/match-001.js";
//     var VERSION = "ECMA_2";
//     var TITLE   = "String.prototype.match( regexp )";

    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;
        
    // the regexp argument is not a RegExp object
    // this is not a string object

    // cases in which the regexp global property is false

//     AddRegExpCases( 3, "3",   "1234567890", 1, 2, ["3"] );

    // cases in which the regexp object global property is true

    AddGlobalRegExpCases( /34/g, "/34/g", "343443444",  3, ["34", "34", "34"] );
    AddGlobalRegExpCases( /\d{1}/g,  "/d{1}/g",  "123456abcde7890", 10,
        ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"] );

    AddGlobalRegExpCases( /\d{2}/g,  "/d{2}/g",  "123456abcde7890", 5,
        ["12", "34", "56", "78", "90"] );

    AddGlobalRegExpCases( /\D{2}/g,  "/d{2}/g",  "123456abcde7890", 2,
        ["ab", "cd"] );

    
    function AddRegExpCases(
        regexp, str_regexp, string, length, index, matches_array ) {
    
        array[item++] = Assert.expectEq(
            "( " + string  + " ).match(" + str_regexp +").length",
            length,
            string.match(regexp).length );
    
        array[item++] = Assert.expectEq(
            "( " + string + " ).match(" + str_regexp +").index",
            index,
            string.match(regexp).index );
    
        array[item++] = Assert.expectEq(
            "( " + string + " ).match(" + str_regexp +").input",
            string,
            string.match(regexp).input );
    
        for ( var matches = 0; matches < matches_array.length; matches++ ) {
            array[item++] = Assert.expectEq(
                "( " + string + " ).match(" + str_regexp +")[" + matches +"]",
                matches_array[matches],
                string.match(regexp)[matches] );
        }
    }
    
    function AddGlobalRegExpCases(
        regexp, str_regexp, string, length, matches_array ) {
    
        array[item++] = Assert.expectEq(
            "( " + string  + " ).match(" + str_regexp +").length",
            length,
            string.match(regexp).length );
    
        for ( var matches = 0; matches < matches_array.length; matches++ ) {
            // adding toString on result... this is ok since we now distinguish
            // between string and object.
            array[item++] = Assert.expectEq(
                "( " + string + " ).match(" + str_regexp +")[" + matches +"]",
                matches_array[matches],
                string.match(regexp)[matches].toString() );
        }
    }
    return array;
}
