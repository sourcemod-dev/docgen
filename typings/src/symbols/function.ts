import { Declaration  } from './base';
import { Argument } from './argument';
import { SearchYield } from './search';
import { Identifier } from './types';

export interface Function extends Declaration {
    tag: Identifier.Function,

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

export interface FunctionYield extends SearchYield {
    readonly tag: Identifier.Function,

    fields: Function,
}

export enum FunctionKind {
    Forward = 'forward',
    Native = 'native',
    Stock = 'stock',
    Static = 'static',
    StaticStock = 'static stock',
    Function = 'function',
}
