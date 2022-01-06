import { YieldType } from './types';

export interface IncludeYield {
    readonly include: string,

    result: SymbolResult[],
}

export interface SymbolResult {
    readonly type: YieldType,

    readonly yields: SymbolYield[],
}

export interface SymbolResultFlatten extends SymbolResult {
    readonly include: string,
}

export interface YieldTypeInclude {
    readonly include: string,

    readonly type: YieldType,
}

export interface SearchYield {
    readonly tag: string,
}

export interface SymbolYield {
    containment: Containment,

    distance: number,

    name: string,
}

export enum Containment {
    InName,
    InParameter,
    InReturn,
}
