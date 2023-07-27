/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 4 -*-
 *
 * ***** BEGIN LICENSE BLOCK *****
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "uc_003";
//     var VERSION = "";
//     var TITLE = "Escapes in identifiers test.";

// TO-DO: commenting the function reference in shell.as
    //printBugNumber (23608);
    //printBugNumber (23607);



    var testcases = getTestCases();
    
function getTestCases()
{
    var array = new Array();
    var item = 0;
    
    var \u0041 = 5;
    var A\u03B2 = 15;
    var c\u0061se = 25;


/*
    array[item++] = Assert.expectEq( "Escaped ASCII Identifier test.", 5, ("\u0041"));
    array[item++] = Assert.expectEq( "Escaped ASCII Identifier test", 6, ("++\u0041"));
    array[item++] = Assert.expectEq( "Escaped non-ASCII Identifier test", 15, ("A\u03B2"));
    array[item++] = Assert.expectEq( "Escaped non-ASCII Identifier test", 16, ("++A\u03B2"));
    array[item++] = Assert.expectEq( "Escaped keyword Identifier test", 25, ("c\\u00" + "61se"));
    array[item++] = Assert.expectEq( "Escaped keyword Identifier test", 26, ("++c\\u00" + "61se"));
 */
                   
    array[item++] = Assert.expectEq( "Escaped ASCII Identifier test.", 5, (\u0041));
    array[item++] = Assert.expectEq( "Escaped ASCII Identifier test", 6, (++\u0041));
    array[item++] = Assert.expectEq( "Escaped non-ASCII Identifier test", 15, (A\u03B2));
    array[item++] = Assert.expectEq( "Escaped non-ASCII Identifier test", 16, (++A\u03B2));
    array[item++] = Assert.expectEq( "Escaped keyword Identifier test", 25, (c\u0061se));
    array[item++] = Assert.expectEq( "Escaped keyword Identifier test", 26, (++c\u0061se));
    
    return array;
}
