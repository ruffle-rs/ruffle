package flash.automation {
    public class ActionGenerator {
        public function ActionGenerator() {
        }

        public function generateAction(action:AutomationAction):void {
            if (action == null) {
                Error.throwError(TypeError, 2007, "action");
            }
        }

        public function generateActions(a:Array):void {
            if (a == null) {
                Error.throwError(TypeError, 2007, "a");
            }
            for each (var action:AutomationAction in a) {
                this.generateAction(action);
            }
        }
    }
}
