import { Declaration  } from './base';
import { Argument } from './argument';

export interface Function extends Declaration {
    /**
     * @brief Function kind (forward, stock, etc)
     * @readonly
     */
    readonly kind: FunctionKind,

    /**
     * @brief Return type of the function
     * @readonly
     */
    readonly returnType: string,

    /**
     * @brief Arguments of the function
     * @readonly
     */
    readonly arguments: Argument[],
}

export enum FunctionKind {
    Forward = 'forward',
    Native = 'native',
    Stock = 'stock',
    Static = 'static',
    StaticStock = 'static stock',
    Function = 'function',
}
