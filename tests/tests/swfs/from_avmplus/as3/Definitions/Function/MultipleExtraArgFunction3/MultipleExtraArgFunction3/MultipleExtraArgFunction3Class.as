/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package MultipleExtraArgFunction3{
	public class MultipleExtraArgFunction3Class implements Inter {
    
    public function returnRest(obj:Object,arr:Array,... rest):Number {
     
        var count = rest.length;
        var a:int = 0;
 
        //print("Output from 3rd test case");
             
        /*for( a = 0; a<count; a++ ){
            print( rest[a] );
        }*/
           
        return count;
     
    }
    
}
}

