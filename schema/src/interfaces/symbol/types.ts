import { Constant } from './constant';
import { Define } from './define';
import { Enumeration } from './enumeration';
import { Function } from './function';
import {
    MethodMap
} from './method_map';
import { Property } from './property';
import { TypeDefinition } from './type_def';
import { TypeSet } from './type_set';
import {
    EnumStruct,
    Field,
} from './enum_struct';

/* eslint-disable no-shadow */
export enum Identifier {
    Constant = 'constant',
    Define = 'define',
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
    Identifier.Define,
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
    Define |
    Enumeration |
    Function |
    MethodMap |
    Property |
    EnumStruct |
    Field |
    TypeDefinition |
    TypeSet;

export type SingletonSymbol = Constant | Define | Function | Enumeration | MethodMap | EnumStruct | TypeDefinition | TypeSet;

export type NestedSymbol = Property | Field;

export const PRIMITIVE_TYPES = [
    'int',
    'int&',
    'int[]',
    'bool',
    'bool&',
    'void',
    'const char[]',
    'char',
    'char[]',
    'const float',
    'float',
    'float&',
    'float[]',
];
