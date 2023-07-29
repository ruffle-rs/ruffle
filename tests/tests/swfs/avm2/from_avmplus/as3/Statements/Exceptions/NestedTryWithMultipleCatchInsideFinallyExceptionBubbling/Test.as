/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import TestPackage.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Testing nested try block with multiple catch blocks, inner try and catch blocks inside finally block of the outer try and multiple catch blocks";  // Provide ECMA section title or a description
var BUGNUMBER = "";



var z:NestedTryWithMultipleCatchInsideFinally = new NestedTryWithMultipleCatchInsideFinally();
thisError = "no error";
thisError1 = "no error";
try{
   z.NestedTryWithMultipleCatchInsideFinallyFunction();
   }catch(eo:ReferenceError){
         thisError=eo.toString();
   }catch(eo1:TypeError){
         thisError="This is outer Type Error:"+eo1.toString();//print(thisError);
   }catch(eo2:ArgumentError){
         thisError=eo2.toString();
   }catch(eo3:URIError){
         thisError=eo3.toString()
   }catch(eo4:UninitializedError){
         thisError=eo4.toString();
   }catch(eo5:EvalError){
         thisError=eo5.toString();
   }catch(eo6:RangeError){
         thisError=eo6.toString();
   }catch(eo7:DefinitionError){
       thisError="This is Definition Error";
   }catch(eo8:SyntaxError){
         thisError="This is Syntax Error";
   }catch(eo9:VerifyError){
         thisError="This is Verify Error";
   }catch(eo10:Error){//print(e10.toString());
         thisError=eo10.toString();
   }finally{//print(eo1.toString());
Assert.expectEq( "Testing catch block with Type Error", "This is outer Type Error:TypeError" ,thisError);
    }




              // displays results.
