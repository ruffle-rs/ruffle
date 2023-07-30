/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "dowhile-007";
//     var VERSION = "ECMA_2";
//     var TITLE   = "do...while";


    var testcases = getTestCases();
    
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    DoWhile( new DoWhileObject( false, false, false, false ));
    DoWhile( new DoWhileObject( true, false, false, false ));
    DoWhile( new DoWhileObject( true, true, false, false ));
    DoWhile( new DoWhileObject( true, true, true, false ));
    DoWhile( new DoWhileObject( true, true, true, true ));
    DoWhile( new DoWhileObject( false, false, false, true ));
    DoWhile( new DoWhileObject( false, false, true, true ));
    DoWhile( new DoWhileObject( false, true, true, true ));
    DoWhile( new DoWhileObject( false, false, true, false ));


    function DoWhileObject( out1, out2, out3, in1 ) {
        this.breakOutOne = out1;
        this.breakOutTwo = out2;
        this.breakOutThree = out3;
        this.breakIn = in1;
    }
    function DoWhile( object ) {
        result1 = false;
        result2 = false;
        result3 = false;
        result4 = false;
    
        outie:
            do {
                if ( object.breakOutOne ) {
                    break outie;
                }
                result1 = true;
    
                innie:
                    do {
                        if ( object.breakOutTwo ) {
                            break outie;
                        }
                        result2 = true;
    
                        if ( object.breakIn ) {
                            break innie;
                        }
                        result3 = true;
    
                    } while ( false );
                        if ( object.breakOutThree ) {
                            break outie;
                        }
                        result4 = true;
            } while ( false );
    
            array[item++] = Assert.expectEq(
                
                "break one: ",
                (object.breakOutOne) ? false : true,
                result1 );
    
            array[item++] = Assert.expectEq(
                
                "break two: ",
                (object.breakOutOne||object.breakOutTwo) ? false : true,
                result2 );
    
            array[item++] = Assert.expectEq(
                
                "break three: ",
                (object.breakOutOne||object.breakOutTwo||object.breakIn) ? false : true,
                result3 );
    
            array[item++] = Assert.expectEq(
                
                "break four: ",
                (object.breakOutOne||object.breakOutTwo||object.breakOutThree) ? false: true,
                result4 );
    }
    return array;
}
