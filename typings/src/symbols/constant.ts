import { Declaration } from './base';
import { SearchYield } from './search';
import { Identifier } from './types';

export interface Constant extends Declaration {
    tag: Identifier.Constant,
}

export interface ConstantYield extends SearchYield {
    readonly tag: Identifier.Constant,

    fields: Constant,
}
