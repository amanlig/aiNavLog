/** Touch opens the service summary; mouse clicks on the initial diagram enter editing. */
export function componentAction(isDraft:boolean,hasSavedDiagram:boolean,isWeb:boolean):'edit-review'|'edit-new'|'show-service' {
  if(isDraft&&isWeb)return 'edit-review';
  if(!hasSavedDiagram&&isWeb)return 'edit-new';
  return 'show-service';
}
