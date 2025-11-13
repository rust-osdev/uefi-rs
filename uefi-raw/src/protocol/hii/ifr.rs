// SPDX-License-Identifier: MIT OR Apache-2.0

//! Internal Forms Representation (IFR) data types

// IFR structs are all packed.
#![allow(missing_debug_implementations)]

use super::{
    AnimationId, DefaultId, FormId, HiiDate, HiiRef, HiiTime, ImageId, QuestionId, StringId,
    VarstoreId,
};
use crate::{Boolean, Guid, newtype_enum};

newtype_enum! {
    /// IFR types (`EFI_IFR_TYPE_*`)
    pub enum IfrType: u8 => {
        NUM_SIZE_8 = 0x00,
        NUM_SIZE_16 = 0x01,
        NUM_SIZE_32 = 0x02,
        NUM_SIZE_64 = 0x03,
        BOOLEAN = 0x04,
        TIME = 0x05,
        DATE = 0x06,
        STRING = 0x07,
        OTHER = 0x08,
        UNDEFINED = 0x09,
        ACTION = 0x0A,
        BUFFER = 0x0B,
        REF = 0x0C,
    }
}

/// `EFI_IFR_TYPE_VALUE`
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub union IfrTypeValue {
    pub u8: u8,           // EFI_IFR_TYPE_NUM_SIZE_8
    pub u16: u16,         // EFI_IFR_TYPE_NUM_SIZE_16
    pub u32: u32,         // EFI_IFR_TYPE_NUM_SIZE_32
    pub u64: u64,         // EFI_IFR_TYPE_NUM_SIZE_64
    pub b: Boolean,       // EFI_IFR_TYPE_BOOLEAN
    pub time: HiiTime,    // EFI_IFR_TYPE_TIME
    pub date: HiiDate,    // EFI_IFR_TYPE_DATE
    pub string: StringId, // EFI_IFR_TYPE_STRING, EFI_IFR_TYPE_ACTION
    pub hii_ref: HiiRef,  // EFI_IFR_TYPE_REF
}

// Compile-time ABI check.
const _: () = {
    assert!(size_of::<IfrTypeValue>() == 22);
    assert!(align_of::<IfrTypeValue>() == 1);
};

impl core::fmt::Debug for IfrTypeValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EfiIfrTypeValue").finish()
    }
}

newtype_enum! {
    /// IFR opcode values (`EFI_IFR_*_OP`)
    pub enum IfrOpcode: u8 => {
        FORM = 0x01,
        SUBTITLE = 0x02,
        TEXT = 0x03,
        IMAGE = 0x04,
        ONE_OF = 0x05,
        CHECKBOX = 0x06,
        NUMERIC = 0x07,
        PASSWORD = 0x08,
        ONE_OF_OPTION = 0x09,
        SUPPRESS_IF = 0x0A,
        LOCKED = 0x0B,
        ACTION = 0x0C,
        RESET_BUTTON = 0x0D,
        FORM_SET = 0x0E,
        REF = 0x0F,
        NO_SUBMIT_IF = 0x10,
        INCONSISTENT_IF = 0x11,
        EQ_ID_VAL = 0x12,
        EQ_ID_ID = 0x13,
        EQ_ID_VAL_LIST = 0x14,
        AND = 0x15,
        OR = 0x16,
        NOT = 0x17,
        RULE = 0x18,
        GRAY_OUT_IF = 0x19,
        DATE = 0x1A,
        TIME = 0x1B,
        STRING = 0x1C,
        REFRESH = 0x1D,
        DISABLE_IF = 0x1E,
        ANIMATION = 0x1F,
        TO_LOWER = 0x20,
        TO_UPPER = 0x21,
        MAP = 0x22,
        ORDERED_LIST = 0x23,
        VARSTORE = 0x24,
        VARSTORE_NAME_VALUE = 0x25,
        VARSTORE_EFI = 0x26,
        VARSTORE_DEVICE = 0x27,
        VERSION = 0x28,
        END = 0x29,
        MATCH = 0x2A,
        GET = 0x2B,
        SET = 0x2C,
        READ = 0x2D,
        WRITE = 0x2E,
        EQUAL = 0x2F,
        NOT_EQUAL = 0x30,
        GREATER_THAN = 0x31,
        GREATER_EQUAL = 0x32,
        LESS_THAN = 0x33,
        LESS_EQUAL = 0x34,
        BITWISE_AND = 0x35,
        BITWISE_OR = 0x36,
        BITWISE_NOT = 0x37,
        SHIFT_LEFT = 0x38,
        SHIFT_RIGHT = 0x39,
        ADD = 0x3A,
        SUBTRACT = 0x3B,
        MULTIPLY = 0x3C,
        DIVIDE = 0x3D,
        MODULO = 0x3E,
        RULE_REF = 0x3F,
        QUESTION_REF1 = 0x40,
        QUESTION_REF2 = 0x41,
        UINT8 = 0x42,
        UINT16 = 0x43,
        UINT32 = 0x44,
        UINT64 = 0x45,
        TRUE = 0x46,
        FALSE = 0x47,
        TO_UINT = 0x48,
        TO_STRING = 0x49,
        TO_BOOLEAN = 0x4A,
        MID = 0x4B,
        FIND = 0x4C,
        TOKEN = 0x4D,
        STRING_REF1 = 0x4E,
        STRING_REF2 = 0x4F,
        CONDITIONAL = 0x50,
        QUESTION_REF3 = 0x51,
        ZERO = 0x52,
        ONE = 0x53,
        ONES = 0x54,
        UNDEFINED = 0x55,
        LENGTH = 0x56,
        DUP = 0x57,
        THIS = 0x58,
        SPAN = 0x59,
        VALUE = 0x5A,
        DEFAULT = 0x5B,
        DEFAULTSTORE = 0x5C,
        FORM_MAP = 0x5D,
        CATENATE = 0x5E,
        GUID = 0x5F,
        SECURITY = 0x60,
        MODAL_TAG = 0x61,
        REFRESH_ID = 0x62,
        WARNING_IF = 0x63,
        MATCH2 = 0x64,
    }
}

/// `EFI_IFR_OP_HEADER`
#[repr(C, packed)]
pub struct IfrOpHeader {
    pub opcode: IfrOpcode,
    pub length_and_scope: u8,
}

/// `EFI_IFR_STATEMENT_HEADER`
#[repr(C, packed)]
pub struct IfrStatementHeader {
    pub prompt: StringId,
    pub help: StringId,
}

#[repr(C, packed)]
pub union VarstoreInfo {
    pub var_name: StringId,
    pub var_offset: u16,
}

bitflags::bitflags! {
    /// `EFI_IFR_FLAG_*`
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[repr(transparent)]
    pub struct IfrQuestionFlags: u8 {
        const READ_ONLY = 1 << 0;
        const CALLBACK = 1 << 2;
        const RESET_REQUIRED = 1 << 4;
        const REST_STYLE = 1 << 5;
        const RECONNECT_REQUIRED = 1 << 6;
        const OPTIONS_ONLY = 1 << 7;
    }
}

/// `EFI_IFR_QUESTION_HEADER`
#[repr(C, packed)]
pub struct IfrQuestionHeader {
    pub header: IfrStatementHeader,
    pub question_id: QuestionId,
    pub varstore_id: VarstoreId,
    pub varstore_info: VarstoreInfo,
    pub flags: IfrQuestionFlags,
}

/// `EFI_IFR_ACTION`
#[repr(C, packed)]
pub struct IfrAction {
    pub header: IfrOpHeader,
    pub question: IfrQuestionHeader,
    pub question_config: StringId,
}

/// `EFI_IFR_ACTION_1`
#[repr(C, packed)]
pub struct IfrAction1 {
    pub header: IfrOpHeader,
    pub question: IfrQuestionHeader,
}

/// `EFI_IFR_ANIMATION`
#[repr(C, packed)]
pub struct IfrAnimation {
    pub header: IfrOpHeader,
    pub id: AnimationId,
}

/// `EFI_IFR_ADD`
#[repr(C, packed)]
pub struct IfrAdd {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_AND`
#[repr(C, packed)]
pub struct IfrAnd {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_BITWISE_AND`
#[repr(C, packed)]
pub struct IfrBitwiseAnd {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_BITWISE_NOT`
#[repr(C, packed)]
pub struct IfrBitwiseNot {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_BITWISE_OR`
#[repr(C, packed)]
pub struct IfrBitwiseOr {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_CATENATE`
#[repr(C, packed)]
pub struct IfrCatenate {
    pub header: IfrOpHeader,
}

pub const IFR_CHECKBOX_DEFAULT: u8 = 0x01;
pub const IFR_CHECKBOX_DEFAULT_MFG: u8 = 0x02;

/// `EFI_IFR_CHECKBOX`
#[repr(C, packed)]
pub struct IfrCheckbox {
    pub header: IfrOpHeader,
    pub question: IfrQuestionHeader,
    pub flags: u8,
}

/// `EFI_IFR_CONDITIONAL`
#[repr(C, packed)]
pub struct IfrConditional {
    pub header: IfrOpHeader,
}

pub const QF_DATE_YEAR_SUPPRESS: u8 = 0x01;
pub const QF_DATE_MONTH_SUPPRESS: u8 = 0x02;
pub const QF_DATE_DAY_SUPPRESS: u8 = 0x04;
pub const QF_DATE_STORAGE: u8 = 0x30;
pub const QF_DATE_STORAGE_NORMAL: u8 = 0x00;
pub const QF_DATE_STORAGE_TIME: u8 = 0x10;
pub const QF_DATE_STORAGE_WAKEUP: u8 = 0x20;

/// `EFI_IFR_DATE`
#[repr(C, packed)]
pub struct IfrDate {
    pub header: IfrOpHeader,
    pub quest: IfrQuestionHeader,
    pub flags: u8,
}

/// `EFI_IFR_DEFAULT`
#[repr(C, packed)]
pub struct IfrDefault {
    pub header: IfrOpHeader,
    pub default_id: u16,
    pub ifr_type: u8,
    pub value: IfrTypeValue,
}

/// `EFI_IFR_DEFAULTSTORE`
#[repr(C, packed)]
pub struct IfrDefaultStore {
    pub header: IfrOpHeader,
    pub default_name: StringId,
    pub default_id: u16,
}

/// `EFI_IFR_DISABLE_IF`
#[repr(C, packed)]
pub struct IfrDisableIf {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_DIVIDE`
#[repr(C, packed)]
pub struct IfrDivide {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_DUP`
#[repr(C, packed)]
pub struct IfrDup {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_END`
#[repr(C, packed)]
pub struct IfrEnd {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_EQUAL`
#[repr(C, packed)]
pub struct IfrEqual {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_EQ_ID_ID`
#[repr(C, packed)]
pub struct IfrEqIdId {
    pub header: IfrOpHeader,
    pub question_id1: QuestionId,
    pub question_id2: QuestionId,
}

/// `EFI_IFR_EQ_ID_VAL_LIST`
#[repr(C, packed)]
pub struct IfrEqIdValList {
    pub header: IfrOpHeader,
    pub question_id: QuestionId,
    pub list_length: u16,
    pub value_list: [u16; 0],
}

/// `EFI_IFR_EQ_ID_VAL`
#[repr(C, packed)]
pub struct IfrEqIdVal {
    pub header: IfrOpHeader,
    pub question_id: QuestionId,
    pub value: u16,
}

/// `EFI_IFR_FALSE`
#[repr(C, packed)]
pub struct IfrFalse {
    pub header: IfrOpHeader,
}

pub const IFR_FF_CASE_SENSITIVE: u8 = 0x00;
pub const IFR_FF_CASE_INSENSITIVE: u8 = 0x01;

/// `EFI_IFR_FIND`
#[repr(C, packed)]
pub struct IfrFind {
    pub header: IfrOpHeader,
    pub format: u8,
}

/// `EFI_IFR_FORM`
#[repr(C, packed)]
pub struct IfrForm {
    pub header: IfrOpHeader,
    pub form_id: FormId,
    pub form_title: StringId,
}

/// `EFI_IFR_FORM_MAP_METHOD`
#[repr(C, packed)]
pub struct IfrFormMapMethod {
    pub method_title: StringId,
    pub method_identifier: Guid,
}

/// `EFI_IFR_FORM_MAP`
#[repr(C, packed)]
pub struct IfrFormMap {
    pub header: IfrOpHeader,
    pub form_id: FormId,
    //pub methods: [IfrFormMapMethod; 0],
}

/// `EFI_IFR_FORM_SET`
#[repr(C, packed)]
pub struct IfrFormSet {
    pub header: IfrOpHeader,
    pub guid: Guid,
    pub formset_title: StringId,
    pub help: StringId,
    pub flags: u8,
    //pub class_guid: [Guid; 0];
}

/// `EFI_IFR_GET`
#[repr(C, packed)]
pub struct IfrGet {
    pub header: IfrOpHeader,
    pub varstore_id: VarstoreId,
    pub varstore_info: VarstoreInfo,
    pub varstore_type: IfrType,
}

/// `EFI_IFR_GRAY_OUT_IF`
#[repr(C, packed)]
pub struct IfrGrayOutIf {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_GREATER_EQUAL`
#[repr(C, packed)]
pub struct IfrGreaterEqual {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_GREATER_THAN`
#[repr(C, packed)]
pub struct IfrGreaterThan {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_GUID`
#[repr(C, packed)]
pub struct IfrGuid {
    pub header: IfrOpHeader,
    pub guid: Guid,
}

/// `EFI_IFR_IMAGE`
#[repr(C, packed)]
pub struct IfrImage {
    pub header: IfrOpHeader,
    pub id: ImageId,
}

/// `EFI_IFR_INCONSISTENT_IF`
#[repr(C, packed)]
pub struct IfrInconsistentIf {
    pub header: IfrOpHeader,
    pub error: StringId,
}

/// `EFI_IFR_LENGTH`
#[repr(C, packed)]
pub struct IfrLength {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_LESS_EQUAL`
#[repr(C, packed)]
pub struct IfrLessEqual {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_LESS_THAN`
#[repr(C, packed)]
pub struct IfrLessThan {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_LOCKED`
#[repr(C, packed)]
pub struct IfrLocked {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_MAP`
#[repr(C, packed)]
pub struct IfrMap {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_MATCH`
#[repr(C, packed)]
pub struct IfrMatch {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_MATCH2`
#[repr(C, packed)]
pub struct IfrMatch2 {
    pub header: IfrOpHeader,
    pub syntax_type: Guid,
}

/// `EFI_IFR_MID`
#[repr(C, packed)]
pub struct IfrMid {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_MODAL_TAG`
#[repr(C, packed)]
pub struct IfrModalTag {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_MODULO`
#[repr(C, packed)]
pub struct IfrModulo {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_MULTIPLY`
#[repr(C, packed)]
pub struct IfrMultiply {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_NOT`
#[repr(C, packed)]
pub struct IfrNot {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_NOT_EQUAL`
#[repr(C, packed)]
pub struct IfrNotEqual {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_NO_SUBMIT_IF`
#[repr(C, packed)]
pub struct IfrNoSubmitIf {
    pub header: IfrOpHeader,
    pub error: StringId,
}

//#[repr(C, packed)]
//pub struct IfrNumericU8 {
//    pub min_value: u8,
//    pub max_value: u8,
//    pub step: u8,
//}
//
//#[repr(C, packed)]
//pub struct IfrNumericU16 {
//    pub min_value: u16,
//    pub max_value: u16,
//    pub step: u16,
//}
//
//#[repr(C, packed)]
//pub struct IfrNumericU32 {
//    pub min_value: u32,
//    pub max_value: u32,
//    pub step: u32,
//}
//
//#[repr(C, packed)]
//pub struct IfrNumericU64 {
//    pub min_value: u64,
//    pub max_value: u64,
//    pub step: u64,
//}
//
//#[repr(C, packed)]
//pub union IfrNumericData {
//    pub ifr_u8: IfrNumericU8,
//    pub ifr_u16: IfrNumericU16,
//    pub ifr_u32: IfrNumericU32,
//    pub ifr_u64: IfrNumericU64,
//}

/// `EFI_IFR_NUMERIC`
#[repr(C, packed)]
pub struct IfrNumeric {
    pub header: IfrOpHeader,
    pub question: IfrQuestionHeader,
    pub flags: u8,
    //pub data: IfrNumericData,
    pub data: [u64; 3], // TODO
}

/// `EFI_IFR_ONE`
#[repr(C, packed)]
pub struct IfrOne {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_ONES`
#[repr(C, packed)]
pub struct IfrOnes {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_ONE_OF`
#[repr(C, packed)]
pub struct IfrOneOf {
    pub header: IfrOpHeader,
    pub question: IfrQuestionHeader,
    pub flags: u8,
    pub data: [u64; 3], // TODO
}

/// `EFI_IFR_ONE_OF_OPTION`
#[repr(C, packed)]
pub struct IfrOneOfOption {
    pub header: IfrOpHeader,
    pub option: StringId,
    pub flags: u8,
    pub ifr_type: IfrType,
    pub value: IfrTypeValue,
}

/// `EFI_IFR_OR`
#[repr(C, packed)]
pub struct IfrOr {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_ORDERED_LIST`
#[repr(C, packed)]
pub struct IfrOrderedList {
    pub header: IfrOpHeader,
    pub question: IfrQuestionHeader,
    pub max_containers: u8,
    pub flags: u8,
}

/// `EFI_IFR_PASSWORD`
#[repr(C, packed)]
pub struct IfrPassword {
    pub header: IfrOpHeader,
    pub question: IfrQuestionHeader,
    pub min_size: u16,
    pub max_size: u16,
}

/// `EFI_IFR_QUESTION_REF1`
#[repr(C, packed)]
pub struct IfrQuestionRef1 {
    pub header: IfrOpHeader,
    pub question_id: QuestionId,
}

/// `EFI_IFR_QUESTION_REF2`
#[repr(C, packed)]
pub struct IfrQuestionRef2 {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_QUESTION_REF3`
#[repr(C, packed)]
pub struct IfrQuestionRef3 {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_QUESTION_REF3_2`
#[repr(C, packed)]
pub struct IfrQuestionRef32 {
    pub header: IfrOpHeader,
    pub device_path: StringId,
}

/// `EFI_IFR_QUESTION_REF3_3`
#[repr(C, packed)]
pub struct IfrQuestionRef33 {
    pub header: IfrOpHeader,
    pub device_path: StringId,
    pub guid: Guid,
}

/// `EFI_IFR_READ`
#[repr(C, packed)]
pub struct IfrRead {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_REF`
#[repr(C, packed)]
pub struct IfrRef {
    pub header: IfrOpHeader,
    pub question: IfrQuestionHeader,
    pub form_id: FormId,
}

/// `EFI_IFR_REF2`
#[repr(C, packed)]
pub struct IfrRef2 {
    pub header: IfrOpHeader,
    pub question: IfrQuestionHeader,
    pub form_id: FormId,
    pub question_id: QuestionId,
}

/// `EFI_IFR_REF3`
#[repr(C, packed)]
pub struct IfrRef3 {
    pub header: IfrOpHeader,
    pub question: IfrQuestionHeader,
    pub form_id: FormId,
    pub question_id: QuestionId,
    pub formset_id: Guid,
}

/// `EFI_IFR_REF4`
#[repr(C, packed)]
pub struct IfrRef4 {
    pub header: IfrOpHeader,
    pub question: IfrQuestionHeader,
    pub form_id: FormId,
    pub question_id: QuestionId,
    pub formset_id: Guid,
    pub device_path: StringId,
}

/// `EFI_IFR_REF5`
#[repr(C, packed)]
pub struct IfrRef5 {
    pub header: IfrOpHeader,
    pub question: IfrQuestionHeader,
}

/// `EFI_IFR_REFRESH`
#[repr(C, packed)]
pub struct IfrRefresh {
    pub header: IfrOpHeader,
    pub refresh_interval: u8,
}

/// `EFI_IFR_REFRESH_ID`
#[repr(C, packed)]
pub struct IfrRefreshId {
    pub header: IfrOpHeader,
    pub refresh_event_group_id: Guid,
}

/// `EFI_IFR_RESET_BUTTON`
#[repr(C, packed)]
pub struct IfrResetButton {
    pub header: IfrOpHeader,
    pub statement: IfrStatementHeader,
    pub default_id: DefaultId,
}

/// `EFI_IFR_RULE`
#[repr(C, packed)]
pub struct IfrRule {
    pub header: IfrOpHeader,
    pub rule_id: u8,
}

/// `EFI_IFR_RULE_REF`
#[repr(C, packed)]
pub struct IfrRuleRef {
    pub header: IfrOpHeader,
    pub rule_id: u8,
}

/// `EFI_IFR_SECURITY`
#[repr(C, packed)]
pub struct IfrSecurity {
    pub header: IfrOpHeader,
    pub permissions: Guid,
}

/// `EFI_IFR_SET`
#[repr(C, packed)]
pub struct IfrSet {
    pub header: IfrOpHeader,
    pub varstore_id: VarstoreId,
    pub varstore_info: VarstoreInfo,
    pub varstore_type: IfrType,
}

/// `EFI_IFR_SHIFT_LEFT`
#[repr(C, packed)]
pub struct IfrShiftLeft {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_SHIFT_RIGHT`
#[repr(C, packed)]
pub struct IfrShiftRight {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_SPAN`
#[repr(C, packed)]
pub struct IfrSpan {
    pub header: IfrOpHeader,
    pub flags: u8,
}

/// `EFI_IFR_STRING`
#[repr(C, packed)]
pub struct IfrString {
    pub header: IfrOpHeader,
    pub question: IfrQuestionHeader,
    pub min_size: u8,
    pub max_size: u8,
    pub flags: u8,
}

/// `EFI_IFR_STRING_REF1`
#[repr(C, packed)]
pub struct IfrStringRef1 {
    pub header: IfrOpHeader,
    pub string_id: StringId,
}

/// `EFI_IFR_STRING_REF2`
#[repr(C, packed)]
pub struct IfrStringRef2 {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_SUBTITLE`
#[repr(C, packed)]
pub struct IfrSubtitle {
    pub header: IfrOpHeader,
    pub statement: IfrStatementHeader,
    pub flags: u8,
}

/// `EFI_IFR_SUBTRACT`
#[repr(C, packed)]
pub struct IfrSubtract {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_SUPPRESS_IF`
#[repr(C, packed)]
pub struct IfrSuppressIf {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_TEXT`
#[repr(C, packed)]
pub struct IfrText {
    pub header: IfrOpHeader,
    pub statement: IfrStatementHeader,
    pub text_two: StringId,
}

/// `EFI_IFR_THIS`
#[repr(C, packed)]
pub struct IfrThis {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_TIME`
#[repr(C, packed)]
pub struct IfrTime {
    pub header: IfrOpHeader,
    pub question: IfrQuestionHeader,
    pub flags: u8,
}

/// `EFI_IFR_TOKEN`
#[repr(C, packed)]
pub struct IfrToken {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_TO_BOOLEAN`
#[repr(C, packed)]
pub struct IfrToBoolean {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_TO_LOWER`
#[repr(C, packed)]
pub struct IfrToLower {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_TO_STRING`
#[repr(C, packed)]
pub struct IfrToString {
    pub header: IfrOpHeader,
    pub format: u8,
}

/// `EFI_IFR_TO_UINT`
#[repr(C, packed)]
pub struct IfrToUint {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_TO_UPPER`
#[repr(C, packed)]
pub struct IfrToUpper {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_TRUE`
#[repr(C, packed)]
pub struct IfrTrue {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_UINT8`
#[repr(C, packed)]
pub struct IfrUint8 {
    pub header: IfrOpHeader,
    pub value: u8,
}

/// `EFI_IFR_UINT16`
#[repr(C, packed)]
pub struct IfrUint16 {
    pub header: IfrOpHeader,
    pub value: u16,
}

/// `EFI_IFR_UINT32`
#[repr(C, packed)]
pub struct IfrUint32 {
    pub header: IfrOpHeader,
    pub value: u32,
}

/// `EFI_IFR_UINT64`
#[repr(C, packed)]
pub struct IfrUint64 {
    pub header: IfrOpHeader,
    pub value: u64,
}

/// `EFI_IFR_UNDEFINED`
#[repr(C, packed)]
pub struct IfrUndefined {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_VALUE`
#[repr(C, packed)]
pub struct IfrValue {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_VARSTORE`
#[repr(C, packed)]
pub struct IfrVarstore {
    pub header: IfrOpHeader,
    pub guid: Guid,
    pub varstore_id: VarstoreId,
    pub size: u16,
    pub name: [u8; 0],
}

/// `EFI_IFR_VARSTORE_NAME_VALUE`
#[repr(C, packed)]
pub struct IfrVarstoreNameValue {
    pub header: IfrOpHeader,
    pub varstore_id: VarstoreId,
    pub guid: Guid,
}

/// `EFI_IFR_VARSTORE_EFI`
#[repr(C, packed)]
pub struct IfrVarstoreEfi {
    pub header: IfrOpHeader,
    pub varstore_id: VarstoreId,
    pub guid: Guid,
    pub attributes: u32,
    pub size: u16,
    pub name: [u8; 0],
}

/// `EFI_IFR_VARSTORE_DEVICE`
#[repr(C, packed)]
pub struct IfrVarstoreDevice {
    pub header: IfrOpHeader,
    pub device_path: StringId,
}

/// `EFI_IFR_VERSION`
#[repr(C, packed)]
pub struct IfrVersion {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_WARNING_IF`
#[repr(C, packed)]
pub struct IfrWarningIf {
    pub header: IfrOpHeader,
    pub warning: StringId,
    pub timeout: u8,
}

/// `EFI_IFR_WRITE`
#[repr(C, packed)]
pub struct IfrWrite {
    pub header: IfrOpHeader,
}

/// `EFI_IFR_ZERO`
#[repr(C, packed)]
pub struct IfrZero {
    pub header: IfrOpHeader,
}
