/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "12.6.3-4";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The for..in statment";
    var BUGNUMBER="http://scopus.mcom.com/bugsplat/show_bug.cgi?id=344855";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    //  for ( LeftHandSideExpression in Expression )
    //  LeftHandSideExpression:NewExpression:MemberExpression

    var o = new MyObject();
    var result = 0;
    var thisError="no error thrown";
  
    for ( MyObject in o ) {
        result += o[MyObject];
    }
   
    
    array[item++] = Assert.expectEq( 
        "for ( MyObject in o ) { result += o[MyObject] }",6,result);
        
    
    var value="value";
    var result = 0;

    for ( value in o ) {
        result += o[value];
    }

    array[item++] = Assert.expectEq( 
        "for ( value in o ) { result += o[value]",
        6,
        result );

    var value = "value";
    var result = 0;
    for ( value in o ) {
        result += o[value];
    }

    array[item++] = Assert.expectEq( 
        "value = \"value\"; for ( value in o ) { result += o[value]",
        6,
        result );

    var value = 0;
    var result = 0;
    for ( value in o ) {
        result += o[value];
    }

    array[item++] = Assert.expectEq( 
        "value = 0; for ( value in o ) { result += o[value]",
        6,
        result );

    // this causes a segv

    var ob = { 0:"hello" };
    var result = 0;
    for ( ob[0] in o ) {
        result += o[ob[0]];
    }
    array[item++] = Assert.expectEq( 
        "ob = { 0:\"hello\" }; for ( ob[0] in o ) { result += o[ob[0]]",
        6,
        result );

    var result = 0;
    for ( ob["0"] in o ) {
        result += o[ob["0"]];
    }
    array[item++] = Assert.expectEq( 
        "value = 0; for ( ob[\"0\"] in o ) { result += o[o[\"0\"]]",
        6,
        result );

    var result = 0;
    var ob = { value:"hello" };
    for ( ob[value] in o ) {
        result += o[ob[value]];
    }
    array[item++] = Assert.expectEq( 
        "ob = { 0:\"hello\" }; for ( ob[value] in o ) { result += o[ob[value]]",
        6,
        result );

    var result = 0;
    for ( ob["value"] in o ) {
        result += o[ob["value"]];
    }
    array[item++] = Assert.expectEq( 
        "value = 0; for ( ob[\"value\"] in o ) { result += o[ob[\"value\"]]",
        6,
        result );

    var result = 0;
    for ( ob.value in o ) {
        result += o[ob.value];
    }
    array[item++] = Assert.expectEq( 
        "value = 0; for ( ob.value in o ) { result += o[ob.value]",
        6,
        result );

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

function MyObject() {
    this.value = 2;
    this[0] = 4;
    return this;
      
}
