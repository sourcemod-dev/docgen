import { IFunction, FunctionKind, IArgument } from '../../interfaces';
import { Declaration } from './base';

export class Function extends Declaration implements IFunction {
    /**
     * @brief Function kind (forward, stock, etc)
     * @readonly
     */
     readonly kind: FunctionKind;

     /**
      * @brief Return type of the function
      * @readonly
      */
     readonly returnType: string;
 
     /**
      * @brief Arguments of the function
      * @readonly
      */
     readonly arguments: IArgument[];

     public constructor(fn: IFunction) {
        super(fn);

        this.kind = fn.kind;
        this.returnType = fn.returnType;
        this.arguments = fn.arguments;
     }
}
