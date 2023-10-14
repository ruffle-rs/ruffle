/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import MultipleCatchBlocksType.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Testing try block with multiple catch blocks and using a catch block with parameter of type TypeError to catch the type error";  // Provide ECMA section title or a description
var BUGNUMBER = "";



var z = new TypeErrors();
thisError = "no error";
thisError1 = "no error";

try{
   z.MyArgumentError("blah");

   }catch(e:ReferenceError){
         thisError=e.toString();
   }catch(e1:TypeError){
         thisError=(e1.toString()).substr(0,22);
   }catch(e2:ArgumentError){
         thisError=e2.toString();
   }catch(e3:URIError){
         thisError=e3.toString()
   }catch(e4:UninitializedError){
         thisError=e4.toString();
   }catch(e5:EvalError){
         thisError=e5.toString();
   }catch(e6:RangeError){
         thisError=e6.toString();
   }catch(e7:DefinitionError){
       thisError="This is Definition Error";
   }catch(e8:SyntaxError){
         thisError="This is Syntax Error";
   }catch(e9:VerifyError){
         thisError="This is Verify Error";
   }catch(e10:Error){//print(e10.toString());
         thisError=e10.toString();
   }finally{
         Assert.expectEq( "Testing catch block with Type Error", "TypeError" ,thisError);
    }




              // displays results.
