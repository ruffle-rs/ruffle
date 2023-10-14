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

//     var SECTION = "";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Unicode Characters 1C-1F negative test";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    // Unicode Characters 1C-1F negative test

    array[item++] = Assert.expectEq(   "Number(00)",        false,
    ("no error" == ('no' + "\u001C" +' error')));
    
    array[item++] = Assert.expectEq(   "Number(01)",        false,
    ("no error" == ('no' + "\u001D" +' error')));
    
    array[item++] = Assert.expectEq(   "Number(02)",        false,
    ("no error" == ('no' + "\u001E" +' error')));
    
    array[item++] = Assert.expectEq(   "Number(03)",        false,
    ("no error" == ('no' + "\u001F" +' error')));
    
    
    return ( array );
}
