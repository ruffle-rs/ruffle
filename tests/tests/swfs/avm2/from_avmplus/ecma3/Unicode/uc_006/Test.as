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

// var SECTION = "";
//     var VERSION = "ECMA_1";
//     var TITLE   = "bug no:118381 High ASCII characters in keywords";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    
    var throwÜ = 100;
    array[item++] = Assert.expectEq(   "var throwÜ = 100",        100,
    (throwÜ));
    
    var classÜ = "string"
    array[item++] = Assert.expectEq(   "var classÜ = 'string'",        "string",
    (classÜ));
    
    var namespaceÜ = false;
    array[item++] = Assert.expectEq(   "var namespaceÜ = false",        false,
    (namespaceÜ));
    
    var asÜ = 1;
    array[item++] = Assert.expectEq(   "var asÜ = 1",        1,
    (asÜ));

    var breakÜ =2;
        array[item++] = Assert.expectEq(   "var breakÜ =2",        2,
    (breakÜ))

    var caseÜ = 3;
        array[item++] = Assert.expectEq(   "var caseÜ = 3",        3,
    (caseÜ))

    var catchÜ = 4;
        array[item++] = Assert.expectEq(   "var catchÜ = 4",        4,
    (catchÜ))
   
    var constÜ = 4;
        array[item++] = Assert.expectEq(   "var constÜ = 4",        4,
    (constÜ))

    var continueÜ = 4;
        array[item++] = Assert.expectEq(   "var continueÜ = 4",        4,
    (continueÜ))

    var deleteÜ = 4;
        array[item++] = Assert.expectEq(   "var deleteÜ = 4",        4,
    (deleteÜ));

    var doÜ = 4;
        array[item++] = Assert.expectEq(   "var doÜ = 4",        4,
    (doÜ));

    var elseÜ = 4;
        array[item++] = Assert.expectEq(   "var elseÜ = 4",        4,
    (elseÜ));

    var extendsÜ = 4;
        array[item++] = Assert.expectEq(   "var extendsÜ = 4",        4,
    (elseÜ));

    var falseÜ = 4;
         array[item++] = Assert.expectEq(   "var falseÜ = 4",        4,
    (falseÜ));

    var finallyÜ = 4;
         array[item++] = Assert.expectEq(   "var finallyÜ = 4",        4,
    (finallyÜ));

    var forÜ = 4;
         array[item++] = Assert.expectEq(   "var forÜ = 4",        4,
    (forÜ));

    var functionÜ = 4;
         array[item++] = Assert.expectEq(   "var functionÜ = 4",        4,
    (functionÜ));

    var ifÜ = 4;
         array[item++] = Assert.expectEq(   "var ifÜ = 4",        4,
    (ifÜ));

    var implementsÜ = 4;
        array[item++] = Assert.expectEq(   "var implementsÜ = 4",        4,
    (implementsÜ));

    var importÜ = 4;
        array[item++] = Assert.expectEq(   "var importÜ = 4",        4,
    (importÜ));

    var inÜ = 4;
        array[item++] = Assert.expectEq(   "var inÜ = 4",        4,
    (inÜ));

    var instanceOfÜ = 4;
        array[item++] = Assert.expectEq(   "var instanceOfÜ = 4",        4,
    (instanceOfÜ));

    var interfaceÜ = 4;
        array[item++] = Assert.expectEq(   "var instanceOfÜ = 4",        4,
    (instanceOfÜ));

    var internalÜ = 4;
        array[item++] = Assert.expectEq(   "var internalÜ = 4",        4,
    (internalÜ));

    var isÜ = 4;
        array[item++] = Assert.expectEq(   "var isÜ = 4",        4,
    (isÜ));

    var nativeÜ = 4;
        array[item++] = Assert.expectEq(   "var nativeÜ = 4",        4,
    (nativeÜ));

    var newÜ = 4;
        array[item++] = Assert.expectEq(   "var newÜ = 4",        4,
    (newÜ));

    var nullÜ = 4;
        array[item++] = Assert.expectEq(   "var nullÜ = 4",        4,
    (nullÜ));

    var packageÜ = 4;
        array[item++] = Assert.expectEq(   "var packageÜ = 4",        4,
    (packageÜ));

    var privateÜ = 4;
        array[item++] = Assert.expectEq(   "var privateÜ = 4",        4,
    (privateÜ));

    var protectedÜ = 4;
        array[item++] = Assert.expectEq(   "var protectedÜ = 4",        4,
    (protectedÜ));

    var publicÜ = 4;
        array[item++] = Assert.expectEq(   "var publicÜ = 4",        4,
    (publicÜ));

    var returnÜ = 4;
        array[item++] = Assert.expectEq(   "var returnÜ = 4",        4,
    (returnÜ));

    var superÜ = 4;
        array[item++] = Assert.expectEq(   "var superÜ = 4",        4,
    (superÜ));

    var switchÜ = 4;
        array[item++] = Assert.expectEq(   "var switchÜ = 4",        4,
    (switchÜ));

    var thisÜ = 4;
        array[item++] = Assert.expectEq(   "var thisÜ = 4",        4,
    (thisÜ));

    var throwÜ = 4;
        array[item++] = Assert.expectEq(   "var throwÜ = 4",        4,
    (throwÜ));

    var toÜ = 4;
        array[item++] = Assert.expectEq(   "var toÜ = 4",        4,
    (toÜ));

    var trueÜ = 4;
        array[item++] = Assert.expectEq(   "var trueÜ = 4",        4,
    (trueÜ));

    var tryÜ = 4;
        array[item++] = Assert.expectEq(   "var tryÜ = 4",        4,
    (tryÜ));

    var typeofÜ = 4;
        array[item++] = Assert.expectEq(   "var typeofÜ = 4",        4,
    (typeofÜ));

     var useÜ = 4;
         array[item++] = Assert.expectEq(   "var useÜ = 4",        4,
    (useÜ));

     var varÜ = 4;
         array[item++] = Assert.expectEq(   "var varÜ = 4",        4,
    (varÜ));

     var voidÜ = 4;
         array[item++] = Assert.expectEq(   "var voidÜ = 4",        4,
    (voidÜ));

     var whileÜ = 4;
         array[item++] = Assert.expectEq(   "var whileÜ = 4",        4,
    (whileÜ));

     var tryÜ = 4;
         array[item++] = Assert.expectEq(   "var tryÜ = 4",        4,
    (tryÜ));

     var eachÜ = 4;
         array[item++] = Assert.expectEq(   "var eachÜ = 4",        4,
    (eachÜ));

     var getÜ = 4;
         array[item++] = Assert.expectEq(   "var getÜ = 4",        4,
    (getÜ));

     var setÜ = 4;
         array[item++] = Assert.expectEq(   "var setÜ = 4",        4,
    (setÜ));

     var namespaceÜ = 4;
         array[item++] = Assert.expectEq(   "var namespaceÜ = 4",        4,
    (namespaceÜ));

     var includeÜ = 4;
         array[item++] = Assert.expectEq(   "var includeÜ = 4",        4,
    (includeÜ));

     var dynamicÜ = 4;
         array[item++] = Assert.expectEq(   "var dynamicÜ = 4",        4,
    (dynamicÜ));

    var finalÜ = 4;
         array[item++] = Assert.expectEq(   "var finalÜ = 4",        4,
    (finalÜ));

    var nativeÜ = 4;
         array[item++] = Assert.expectEq(   "var nativeÜ = 4",        4,
    (nativeÜ))

    var overrideÜ = 4;
         array[item++] = Assert.expectEq(   "var overrideÜ = 4",        4,
    (overrideÜ))

    var staticÜ = 4;
         array[item++] = Assert.expectEq(   "var staticÜ = 4",        4,
    (staticÜ))

    
    
    return ( array );
}
