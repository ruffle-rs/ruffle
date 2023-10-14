/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "8.6.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Property attributes of Object type";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var OBJ_PROT = Object.prototype;

    try{
        Object.prototype=null
    }catch(e:ReferenceError){
        thisError=e.toString();
    }finally{
        array[item++]=Assert.expectEq("Verifying the read only property of Object.prototype","ReferenceError: Error #1074",Utils.referenceError(thisError));
    }
    array[item++] = Assert.expectEq(   
                                    "var OBJ_PROT = Object.prototype; Object.prototype = null; Object.prototype == OBJ_PROT",
                                    true,
                                    (OBJ_PROT = Object.prototype, Object.prototype == OBJ_PROT ) );
    

    try{
        Object.prototype=0
    }catch(e:ReferenceError){
        thisError=e.toString();
    }finally{
        array[item++]=Assert.expectEq("Verifying the read only property of Object.prototype","ReferenceError: Error #1074",Utils.referenceError(thisError));
    }
    array[item++] = Assert.expectEq(   
                                    "Object.prototype=0; Object.prototype",
                                    Object.prototype,
                                    Object.prototype );

    var OBJ_PROT1 = Object.prototype;
    delete( Object.prototype );
    array[item++] = Assert.expectEq( "var OBJ_PROT1 = Object.prototype; delete( Object.prototype ); OBJ_PROT1 == Object.prototype",    true, OBJ_PROT1 == Object.prototype);
    array[item++] = Assert.expectEq( "delete( Object.prototype )",          false,       delete( Object.prototype ) );

    var string = '';
    for ( prop in Object ) {
        string += ( prop == 'prototype' ) ? prop : '';
    }

    array[item++] = Assert.expectEq("var string = ''; for ( prop in Object ) { string += ( prop == 'prototype' ) ? prop: '' } string;","",string);

    

    return ( array );
}


function MyObject( value ) {
    this.value = value;

    // the new Function() changes to function() {}.
    this.valueOf = function() { return this.value; }
    this.toString = function() { return this.value+''; }
    this.valueOf = function() { return this.value; }
    this.toString = function() { return this.value +'';}
}
