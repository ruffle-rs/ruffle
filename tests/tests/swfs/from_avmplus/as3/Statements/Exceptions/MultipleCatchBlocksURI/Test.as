/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import MultipleCatchBlocksURI.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Testing try block with multiple catch blocks";  // Provide ECMA section title or a description
var BUGNUMBER = "";



var z = new URIErrors();
thisError = "no error";
       
try{
   z.MyArgumentError(6);
   }catch(e:ReferenceError){
         thisError=e.toString();
   }catch(e1:TypeError){
         thisError=e1.toString();
   }catch(e2:ArgumentError){
         thisError=e2.toString();
   }catch(e3:URIError){
         thisError=(e3.toString()).substr(0,21);
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
   }catch(e10:Error){
         thisError=e7.toString();
   }finally{
         Assert.expectEq( "Testing catch block with URI Error", "URIError: Error #1052",thisError);
    }

              // displays results.
