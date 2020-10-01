import { Constant, ConstantYield } from './constant';
import { Enumeration, EnumerationYield } from './enumeration';
import { Function, FunctionYield } from './function';
import {
    MethodMap, MethodMapYield, MethodMapMethodYield, MethodMapPropertyYield,
} from './method_map';
import { Property } from './property';
import { TypeDefinition, TypeDefinitionYield } from './type_def';
import { TypeSet, TypeSetYield } from './type_set';
import {
    EnumStruct,
    Field,
    EnumStructYield,
    EnumStructMethodYield,
    EnumStructFieldYield,
} from './enum_struct';

/* eslint-disable no-shadow */
export enum Identifier {
    Constant = 'constant',
    Enumeration = 'enumeration',
    Function = 'function',
    MethodMap = 'methodmap',
    MethodMapMethod = 'methodmap_method',
    MethodMapProperty = 'methodmap_property',
    EnumStruct = 'enumstruct',
    EnumStructMethod = 'enumstruct_method',
    EnumStructField = 'enumstruct_field',
    Field = 'field',
    Property = 'property',
    TypeDefinition = 'typedef',
    TypeSet = 'typeset',
}

// Weights smaller than 0.1
export enum IdentifierWeights {
    Enumeration = 0.03,
    Function = 0.02,
    MethodMap = 0.05,
    EnumStruct = 0.05,
    TypeDefinition = 0.03,
    TypeSet = 0.03,
}

export const SINGLETON_TYPES = [
    Identifier.Constant,
    Identifier.Function,
    Identifier.Enumeration,
    Identifier.MethodMap,
    Identifier.EnumStruct,
    Identifier.TypeDefinition,
    Identifier.TypeSet,
];

export const NESTED_TYPES = [
    Identifier.MethodMapMethod,
    Identifier.MethodMapProperty,
    Identifier.EnumStructMethod,
    Identifier.EnumStructField,
];

export type Symbol =
    Constant |
    Enumeration |
    Function |
    MethodMap |
    Property |
    EnumStruct |
    Field |
    TypeDefinition |
    TypeSet;

export type SingletonYield =
    | ConstantYield
    | FunctionYield
    | EnumerationYield
    | MethodMapYield
    | EnumStructYield
    | TypeSetYield
    | TypeDefinitionYield;

export type NestedYield =
    | MethodMapMethodYield
    | MethodMapPropertyYield
    | EnumStructMethodYield
    | EnumStructFieldYield;

export type YieldType =
    | ConstantYield
    | FunctionYield
    | EnumerationYield
    | MethodMapYield
    | MethodMapMethodYield
    | MethodMapPropertyYield
    | EnumStructYield
    | EnumStructMethodYield
    | EnumStructFieldYield
    | TypeSetYield
    | TypeDefinitionYield;
