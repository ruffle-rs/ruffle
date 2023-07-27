/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;


//     var SECTION = "e11_4_1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The delete operator";
    var obj = new Object();


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;
    var x;

    x=[9,8,7]; delete x[2];
    var len =  x.length;
    array[item++] = Assert.expectEq(    "x=[9,8,7];delete(x[2]);x.length",         3,            len );
    var str =  x.toString();
    array[item++] = Assert.expectEq(    "x=[9,8,7];delete(x[2]);x.toString()",     "9,8,",        str );
        var obj = new Object();
    obj.name="Jeffrey";
    delete obj.name;
    array[item++] = Assert.expectEq(    "obj=new Object();delete obj.name;",        undefined,    obj.name);
    // the object obj should be deletable but failed!
    delete obj;
    array[item++] = Assert.expectEq(    "obj=new Object();delete obj;obj.toString()",        "[object Object]",    obj.toString());

    //array[item++] = Assert.expectEq(    "delete('string primitive')",   true,   delete("string primitive") );
    array[item++] = Assert.expectEq(    "delete(new String( 'string object' ) )",  true,   delete(new String("string object")) );
    array[item++] = Assert.expectEq(    "delete(new Number(12345))",  true,   delete(new Number(12345)) );
    array[item++] = Assert.expectEq(    "delete(Math.PI)",             false,   delete(Math.PI) );
    //array[item++] = Assert.expectEq(    "delete null ",                true,   delete null );
    //array[item++] = Assert.expectEq(    "delete(void(0))",             true,   delete(void(0)) );

    // variables declared with the var statement are not deletable
    array[item++] = Assert.expectEq(    "delete(x=new Date())",        true,   delete(x=new Date()) );
    var abc;
    array[item++] = Assert.expectEq(    "var abc; delete abc",        false,   delete abc );
    var OB = new MyObject();
    for ( p in OB ) {
        array[item++] = Assert.expectEq(   
                                    "var OB = new MyObject(); for ( p in OB ) { delete OB[p] }",
                                    true, delete OB[p]  );
        //trace("after delete: p = "+p+", OB[p] = "+OB[p]);
    }
    delete OB;
    array[item++] = Assert.expectEq(    "var OB = new MyObject();delete OB; OB.toString()",        "[object Object]",    OB.toString());
    return ( array );
}

function MyObject() {
    this.prop1 = true;
    this.prop2 = false;
    this.prop3 = null
    this.prop4 = void 0;
    this.prop5 = "hi";
    this.prop6 = 42;
    this.prop7 = new Date();
    this.prop8 = Math.PI;
}
