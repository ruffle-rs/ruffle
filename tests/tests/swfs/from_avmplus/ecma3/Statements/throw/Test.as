/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = '12.13';
//     var VERSION = 'no version';
//     var TITLE = 'Statement:throw';



    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;
    
    var t;
    var z = 1;
    

    try {
         if(z == 1)
            throw "Error 1"
         else if(z == 2)
            throw "Error 2"
        }
    catch(er) {
        if(er == "Error 1")
        t = "Error 1"
        if(er == "Error 2")
        t = "Error 2"
}
            
    
    array[item++] = Assert.expectEq(  "throw t", "Error 1", t);

       var z = 2;
       var Exception_Value = "no error";

       
       
       function f(Exception_Value){
           if (z==2){
           this.Exception_Value = Exception_Value;
           throw Exception_Value;
           }else{
                return Exception_Value
           }

       }
       thisError='No Error'
       var Err_msg = "Error!z is equal to 2"
       try{
           f(Err_msg);
       }catch(e){
           thisError=e.toString();
       }finally{
      array[item++] = Assert.expectEq(  "throw t", "Error!z is equal to 2", thisError);
       }

    return array;
}

