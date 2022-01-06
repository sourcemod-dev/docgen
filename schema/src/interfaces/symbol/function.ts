import { IDeclaration  } from './base';
import { IArgument } from './argument';

export interface IFunction extends IDeclaration {
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
}

export enum FunctionKind {
    Forward = 'forward',
    Native = 'native',
    Stock = 'stock',
    Static = 'static',
    StaticStock = 'static stock',
    Function = 'function',
}
