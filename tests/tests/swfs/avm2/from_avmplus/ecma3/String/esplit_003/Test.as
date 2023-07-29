/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

/*
 * Since regular expressions have been part of JavaScript since 1.2, there
 * are already tests for regular expressions in the js1_2/regexp folder.
 *
 * These new tests try to supplement the existing tests, and verify that
 * our implementation of RegExp conforms to the ECMA specification, but
 * does not try to be as exhaustive as in previous tests.
 *
 * The [,limit] argument to String.split is new, and not covered in any
 * existing tests.
 *
 * String.split cases are covered in ecma/String/15.5.4.8-*.js.
 * String.split where separator is a RegExp are in
 * js1_2/regexp/string_split.js
 *
 */

//     var SECTION = "ecma_2/String/split-003.js";
//     var VERSION = "ECMA_2";
//     var TITLE   = "String.prototype.split( regexp, [,limit] )";


    // separartor is a regexp
    // separator regexp value global setting is set
    // string is an empty string
    // if separator is an empty string, split each by character

    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    AddSplitCases( "hello", new RegExp, "new RegExp", ["h","e","l","l","o"] );

    AddSplitCases( "hello", /l/, "/l/", ["he","","o"] );
    AddLimitedSplitCases( "hello", /l/, "/l/", 0, [] );
    AddLimitedSplitCases( "hello", /l/, "/l/", 1, ["he"] );
    AddLimitedSplitCases( "hello", /l/, "/l/", 2, ["he",""] );
    AddLimitedSplitCases( "hello", /l/, "/l/", 3, ["he","","o"] );
    AddLimitedSplitCases( "hello", /l/, "/l/", 4, ["he","","o"] );
    AddLimitedSplitCases( "hello", /l/, "/l/", void 0, ["he","","o"] );
    AddLimitedSplitCases( "hello", /l/, "/l/", "hi", [] );
    AddLimitedSplitCases( "hello", /l/, "/l/", undefined, ["he","","o"] );

    AddSplitCases( "hello", new RegExp, "new RegExp", ["h","e","l","l","o"] );
    AddLimitedSplitCases( "hello", new RegExp, "new RegExp", 0, [] );
    AddLimitedSplitCases( "hello", new RegExp, "new RegExp", 1, ["h"] );
    AddLimitedSplitCases( "hello", new RegExp, "new RegExp", 2, ["h","e"] );
    AddLimitedSplitCases( "hello", new RegExp, "new RegExp", 3, ["h","e","l"] );
    AddLimitedSplitCases( "hello", new RegExp, "new RegExp", 4, ["h","e","l","l"] );
    AddLimitedSplitCases( "hello", new RegExp, "new RegExp", void 0,  ["h","e","l","l","o"] );
    AddLimitedSplitCases( "hello", new RegExp, "new RegExp", "hi",  [] );
    AddLimitedSplitCases( "hello", new RegExp, "new RegExp", undefined,  ["h","e","l","l","o"] );


    function AddSplitCases( string, separator, str_sep, split_array ) {
        // verify that the result of split is an object of type Array
        array[item++] = Assert.expectEq(
            "( " + string  + " ).split(" + str_sep +").constructor == Array",
            true,
            string.split(separator).constructor == Array );
    
        // check the number of items in the array
        array[item++] = Assert.expectEq(
            "( " + string  + " ).split(" + str_sep +").length",
            split_array.length,
            string.split(separator).length );
    
        // check the value of each array item
        var limit = (split_array.length > string.split(separator).length )
            ? split_array.length : string.split(separator).length;
    
        for ( var matches = 0; matches < split_array.length; matches++ ) {
            array[item++] = Assert.expectEq(
                "( " + string + " ).split(" + str_sep +")[" + matches +"]",
                split_array[matches],
                string.split( separator )[matches] );
        }
    }
    
    function AddLimitedSplitCases(
        string, separator, str_sep, limit, split_array ) {
    
        // verify that the result of split is an object of type Array
    
        array[item++] = Assert.expectEq(
            "( " + string  + " ).split(" + str_sep +", " + limit +
                " ).constructor == Array",
            true,
            string.split(separator, limit).constructor == Array );
    
        // check the length of the array
    
        array[item++] = Assert.expectEq(
            "( " + string + " ).split(" + str_sep  +", " + limit + " ).length",
            split_array.length,
            string.split(separator, limit).length );
    
        // check the value of each array item
    
        var slimit = (split_array.length > string.split(separator).length )
            ? split_array.length : string.split(separator, limit).length;
    
        for ( var matches = 0; matches < slimit; matches++ ) {
            array[item++] = Assert.expectEq(
                "( " + string + " ).split(" + str_sep +", " + limit + " )[" + matches +"]",
                split_array[matches],
                string.split( separator, limit )[matches] );
        }
    }
    
    return array;
}
