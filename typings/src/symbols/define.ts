import { Declaration } from './base';
import { SearchYield } from './search';
import { Identifier } from './types';

export interface Define extends Declaration {
    tag: Identifier.Define,
}

export interface DefineYield extends SearchYield {
    readonly tag: Identifier.Constant,

    fields: Define,
}
