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

//     var SECTION = "String/match-003.js";
//     var VERSION = "ECMA_2";
//     var TITLE   = "String.prototype.match( regexp )";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;
        
    // the regexp argument is not a RegExp object
    // this is not a string object


//  [if regexp.global is true] set the regexp.lastIndex property to 0 and
//  invoke RegExp.prototype.exec repeatedly until there is no match. If
//  there is a match with an empty string (in other words, if the value of
//  regexp.lastIndex is left unchanged) increment regexp.lastIndex by 1.
//  The value returned is an array with the properties 0 through n-1
//  corresponding to the first element of the result of each matching invocation
//  of RegExp.prototype.exec.


    // set the value of lastIndex
    re = /([\d]{5})([-\ ]?[\d]{4})?$/g;


    s = "Boston, MA 02134";

    AddGlobalRegExpCases( re,
                          "re = " + re,
                          s,
                          ["02134" ]);

    re.lastIndex = 0;

    AddGlobalRegExpCases(
                     re,
                     "re = " + re + "; re.lastIndex = 0 ",
                     s,
                     ["02134"]);


    re.lastIndex = s.length;

    AddGlobalRegExpCases(
                    re,
                    "re = " + re + "; re.lastIndex = " + s.length,
                    s,
                    ["02134"] );

    re.lastIndex = s.lastIndexOf("0");

    AddGlobalRegExpCases(
                    re,
                    "re = "+ re +"; re.lastIndex = " + s.lastIndexOf("0"),
                    s,
                    ["02134"]);

    re.lastIndex = s.lastIndexOf("0") + 1;

    AddGlobalRegExpCases(
                    re,
                    "re = " +re+ "; re.lastIndex = " + (s.lastIndexOf("0") +1),
                    s,
                    ["02134"]);


    function AddGlobalRegExpCases(
        regexp, str_regexp, string, matches_array ) {
    
      // prevent a runtime error
    
        if ( string.match(regexp) == null || matches_array == null ) {
            array[item++] = Assert.expectEq(
              string + ".match(" + str_regexp +")",
              matches_array,
              string.match(regexp) );
    
            return;
        }
    
        array[item++] = Assert.expectEq(
            "( " + string  + " ).match(" + str_regexp +").length",
            matches_array.length,
            string.match(regexp).length );
    
        var limit = matches_array.length > string.match(regexp).length ?
                    matches_array.length :
                    string.match(regexp).length;
    
        for ( var matches = 0; matches < limit; matches++ ) {
            array[item++] = Assert.expectEq(
                "( " + string + " ).match(" + str_regexp +")[" + matches +"]",
                matches_array[matches],
                string.match(regexp)[matches] );
        }
    }
    return array;
}
