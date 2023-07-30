/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

 

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import MultipleCatchBlocks2.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Testing try block with multiple catch blocks";  // Provide ECMA section title or a description
var BUGNUMBER = "";



var z = new MyErrors2();
thisError = "no error";
thisError1 = "no error";
thisError2 = "no error";
thisError3 = "no error";
thisError4 = "no error";
thisError5 = "no error";
thisError6 = "no error";
thisError7 = "no error";
try {
    z.MyArgumentError();
    }catch(e:ReferenceError){
          thisError=(e.toString()).substr(0,26);//print(thisError);
    }catch(e1:TypeError){
          thisError1=(e1.toString()).substr(0,26);//print(thisError1);
    }catch(e2:ArgumentError){
          thisError2=(e2.toString()).substr(0,26);//print(thisError2);
    }catch(e3:URIError){
          thisError3=(e3.toString()).substr(0,26);//print(thisError3);
    }catch(e4:UninitializedError){
          thisError4=(e4.toString()).substr(0,26);//print(thisError4);
    }catch(e5:EvalError){
          thisError5=(e5.toString()).substr(0,26);//print(thisError5);
    }catch(e6:RangeError){
          thisError6=(e6.toString()).substr(0,26);//print(thisError6);
    }catch(e7:Error){
          thisError7=(e7.toString()).substr(0,26);//print(thisError7);
    }finally{
Assert.expectEq( "Testing catch block with Reference Error", "no error" ,thisError);
Assert.expectEq( "Testing catch block with Type Error", "no error" ,thisError1);
Assert.expectEq( "Testing catch block with Argument Error", "ArgumentError: Error #1063",thisError2);
Assert.expectEq( "Testing catch block with URIError", "no error" ,thisError3);
Assert.expectEq( "Testing catch block with Eval Error", "no error" ,thisError5);
Assert.expectEq( "Testing catch block with Range Error", "no error" ,thisError6);
Assert.expectEq( "Testing catch block with Error", "no error" ,thisError7);
            }

              // displays results.
