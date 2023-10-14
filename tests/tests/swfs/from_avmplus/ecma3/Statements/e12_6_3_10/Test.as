/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "12.6.3-10";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The for..in statment";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    //  for ( LeftHandSideExpression in Expression )
    //  LeftHandSideExpression:NewExpression:MemberExpression

    var count = 0;
    function f() {
        //print( "f called" );
        count++;
        return new Array("h","e","l","o");
    }

    var myArray = new Array("h","e","l","o");

    var result = "";
    for ( p in f() ) { result += f()[p] };

    array[item++] = Assert.expectEq( 
        "count = 0; result = \"\"; "+
        "function f() { count++; return new Array(\"h\",\"e\",\"l\",\"o\"); }"+
        "for ( p in f() ) { result += f()[p] }; count",
        5,
        count );

    // ecma does not gaurantee the order that for in will run... changed
    // to verify that all letters of hello are called
    var result2 = "PASSED";
    for( var x = 0; x<myArray.length; x++ ){
        if( result.indexOf( myArray[x] ) == -1 ){
            result2 = "FAILED";
            break;
        }
    }

    array[item++] = Assert.expectEq( 
        "Verify all letters of hello are found in result",
        "PASSED",
        result2 );

    //  LeftHandSideExpression:NewExpression:MemberExpression [ Expression ]
    //  LeftHandSideExpression:NewExpression:MemberExpression . Identifier
    //  LeftHandSideExpression:NewExpression:new MemberExpression Arguments
    //  LeftHandSideExpression:NewExpression:PrimaryExpression:( Expression )
    //  LeftHandSideExpression:CallExpression:MemberExpression Arguments
    //  LeftHandSideExpression:CallExpression Arguments
    //  LeftHandSideExpression:CallExpression [ Expression ]
    //  LeftHandSideExpression:CallExpression . Identifier

    return array;
}
